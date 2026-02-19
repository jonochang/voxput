// Voxput GNOME Shell Extension
// Requires: voxputd running on D-Bus as com.github.jonochang.Voxput
// Compatible with GNOME Shell 45+

import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import St from 'gi://St';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';

import { Extension, gettext as _ } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

// ---------------------------------------------------------------------------
// D-Bus interface XML for the voxputd service
// GetStatus returns 3 separate string out-args (zbus Rust tuple → 3 × "s")
// ---------------------------------------------------------------------------

const DBUS_IFACE = `
<node>
  <interface name="com.github.jonochang.Voxput1">
    <method name="StartRecording"/>
    <method name="StopRecording"/>
    <method name="Toggle"/>
    <method name="GetStatus">
      <arg type="s" direction="out" name="state"/>
      <arg type="s" direction="out" name="transcript"/>
      <arg type="s" direction="out" name="error"/>
    </method>
    <signal name="StateChanged">
      <arg type="s" name="state"/>
      <arg type="s" name="transcript"/>
    </signal>
  </interface>
</node>`;

const DBUS_BUS_NAME    = 'com.github.jonochang.Voxput';
const DBUS_OBJECT_PATH = '/com/github/jonochang/Voxput';

// ---------------------------------------------------------------------------
// Icon names per state
// ---------------------------------------------------------------------------

const STATE_ICONS = {
    idle:          'audio-input-microphone-muted-symbolic',
    recording:     'audio-input-microphone-symbolic',
    transcribing:  'emblem-synchronizing-symbolic',
    error:         'dialog-error-symbolic',
};

const STATE_STYLE_CLASSES = {
    idle:          'voxput-idle',
    recording:     'voxput-recording',
    transcribing:  'voxput-transcribing',
    error:         'voxput-error',
};

// ---------------------------------------------------------------------------
// Main extension class
// ---------------------------------------------------------------------------

export default class VoxputExtension extends Extension {
    enable() {
        this._settings = this.getSettings();
        this._state = 'idle';
        this._lastTranscript = '';
        this._proxy = null;
        this._signalId = null;
        this._nameWatchId = null;
        this._keyReleaseId = null;

        this._buildIndicator();
        this._connectDbus();
        this._bindShortcut();
    }

    disable() {
        this._unbindShortcut();
        this._cleanupKeyRelease();
        this._disconnectDbus();
        this._destroyIndicator();
        this._settings = null;
    }

    // -----------------------------------------------------------------------
    // Indicator (top-bar panel button)
    // -----------------------------------------------------------------------

    _buildIndicator() {
        this._indicator = new PanelMenu.Button(0.0, this.metadata.name, false);

        this._icon = new St.Icon({
            icon_name: STATE_ICONS['idle'],
            style_class: `system-status-icon voxput-icon ${STATE_STYLE_CLASSES['idle']}`,
        });
        this._indicator.add_child(this._icon);

        const menu = this._indicator.menu;

        this._statusItem = new PopupMenu.PopupMenuItem(_('Idle'), { reactive: false });
        menu.addMenuItem(this._statusItem);

        this._transcriptItem = new PopupMenu.PopupMenuItem('', { reactive: false });
        this._transcriptItem.label.clutter_text.set_line_wrap(true);
        this._transcriptItem.visible = false;
        menu.addMenuItem(this._transcriptItem);

        menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        this._toggleItem = new PopupMenu.PopupMenuItem(_('Start Recording'));
        this._toggleItem.connect('activate', () => this._toggle());
        menu.addMenuItem(this._toggleItem);

        const prefsItem = new PopupMenu.PopupMenuItem(_('Settings'));
        prefsItem.connect('activate', () => this.openPreferences());
        menu.addMenuItem(prefsItem);

        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    _destroyIndicator() {
        this._indicator?.destroy();
        this._indicator = null;
    }

    _updateIndicator(state, transcript) {
        this._state = state;
        if (transcript)
            this._lastTranscript = transcript;

        const iconName   = STATE_ICONS[state]        ?? STATE_ICONS['idle'];
        const styleClass = STATE_STYLE_CLASSES[state] ?? STATE_STYLE_CLASSES['idle'];

        this._icon.icon_name = iconName;
        for (const cls of Object.values(STATE_STYLE_CLASSES))
            this._icon.remove_style_class_name(cls);
        this._icon.add_style_class_name(styleClass);

        const labels = {
            idle:         _('Idle'),
            recording:    _('Recording…'),
            transcribing: _('Transcribing…'),
            error:        _('Error'),
        };
        this._statusItem.label.text = labels[state] ?? state;

        if (state === 'idle' && this._lastTranscript) {
            const short = this._lastTranscript.length > 80
                ? this._lastTranscript.slice(0, 80) + '…'
                : this._lastTranscript;
            this._transcriptItem.label.text = short;
            this._transcriptItem.visible = true;
        } else if (state !== 'idle') {
            this._transcriptItem.visible = false;
        }

        this._toggleItem.label.text =
            state === 'recording' ? _('Stop Recording') : _('Start Recording');
    }

    // -----------------------------------------------------------------------
    // D-Bus connection
    // -----------------------------------------------------------------------

    _connectDbus() {
        const VoxputProxy = Gio.DBusProxy.makeProxyWrapper(DBUS_IFACE);
        try {
            this._proxy = new VoxputProxy(
                Gio.DBus.session,
                DBUS_BUS_NAME,
                DBUS_OBJECT_PATH,
                null,
                Gio.DBusProxyFlags.NONE,
            );

            this._signalId = this._proxy.connectSignal(
                'StateChanged',
                (_proxy, _sender, [state, transcript]) => {
                    this._onStateChanged(state, transcript);
                },
            );

            this._refreshStatus();

            // Detect daemon appearing / disappearing
            this._nameWatchId = Gio.DBus.session.watch_name(
                DBUS_BUS_NAME,
                Gio.BusNameWatcherFlags.NONE,
                () => this._refreshStatus(),
                () => this._updateIndicator('idle', ''),
            );

            if (this._settings.get_boolean('daemon-auto-start'))
                this._ensureDaemonRunning();

        } catch (e) {
            logError(e, 'Voxput: failed to connect to D-Bus');
        }
    }

    _disconnectDbus() {
        if (this._nameWatchId) {
            Gio.DBus.session.unwatch_name(this._nameWatchId);
            this._nameWatchId = null;
        }
        if (this._proxy && this._signalId) {
            this._proxy.disconnectSignal(this._signalId);
            this._signalId = null;
        }
        this._proxy = null;
    }

    _onStateChanged(state, transcript) {
        this._updateIndicator(state, transcript);

        if (state === 'idle' && transcript &&
            this._settings.get_boolean('show-transcript-notification')) {
            Main.notify(_('Voxput'), transcript);
        }
        if (state === 'error') {
            Main.notifyError(_('Voxput'), _('Recording or transcription failed.'));
        }
    }

    _refreshStatus() {
        if (!this._proxy)
            return;
        // GetStatus returns 3 separate string args: [state, transcript, error]
        this._proxy.GetStatusRemote((result, error) => {
            if (error || !result)
                return;
            const [state, transcript, _err] = result;
            this._updateIndicator(state, transcript);
        });
    }

    // -----------------------------------------------------------------------
    // Actions
    // -----------------------------------------------------------------------

    _toggle() {
        if (!this._proxy) return;
        this._proxy.ToggleRemote((_result, error) => {
            if (error)
                logError(error, 'Voxput: toggle failed');
        });
    }

    _startPushToTalk() {
        if (!this._proxy) return;
        // Guard against key-repeat firing this multiple times
        if (this._keyReleaseId) return;

        this._proxy.StartRecordingRemote((_result, error) => {
            if (error)
                logError(error, 'Voxput: start recording failed');
        });

        // Stop on the first key-release event (any key)
        this._keyReleaseId = global.stage.connect('key-release-event', () => {
            this._stopPushToTalk();
            return false; // propagate the event
        });
    }

    _stopPushToTalk() {
        this._cleanupKeyRelease();
        if (!this._proxy) return;
        this._proxy.StopRecordingRemote((_result, error) => {
            if (error)
                logError(error, 'Voxput: stop recording failed');
        });
    }

    _cleanupKeyRelease() {
        if (this._keyReleaseId) {
            global.stage.disconnect(this._keyReleaseId);
            this._keyReleaseId = null;
        }
    }

    _ensureDaemonRunning() {
        // Triggers D-Bus activation; voxputd starts if not already running.
        this._proxy?.GetStatusRemote(() => {});
    }

    // -----------------------------------------------------------------------
    // Keyboard shortcut (push-to-talk: hold to record, release to stop)
    // -----------------------------------------------------------------------

    _bindShortcut() {
        Main.wm.addKeybinding(
            'toggle-recording',
            this._settings,
            Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
            Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
            () => this._startPushToTalk(),
        );
    }

    _unbindShortcut() {
        Main.wm.removeKeybinding('toggle-recording');
    }
}
