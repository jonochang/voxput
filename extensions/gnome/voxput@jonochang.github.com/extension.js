// Voxput GNOME Shell Extension
// Requires: voxputd running on D-Bus as com.github.jonochang.Voxput
// Compatible with GNOME Shell 45+

import GLib from 'gi://GLib';
import Gio from 'gi://Gio';
import St from 'gi://St';
import Clutter from 'gi://Clutter';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';

import { Extension, gettext as _ } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as MessageTray from 'resource:///org/gnome/shell/ui/messageTray.js';

// ---------------------------------------------------------------------------
// D-Bus interface XML for the voxputd service
// ---------------------------------------------------------------------------

const DBUS_IFACE = `
<node>
  <interface name="com.github.jonochang.Voxput1">
    <method name="StartRecording"/>
    <method name="StopRecording"/>
    <method name="Toggle"/>
    <method name="GetStatus">
      <arg type="(sss)" direction="out" name="result"/>
    </method>
    <signal name="StateChanged">
      <arg type="s" name="state"/>
      <arg type="s" name="transcript"/>
    </signal>
  </interface>
</node>`;

const DBUS_BUS_NAME = 'com.github.jonochang.Voxput';
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

// CSS style class applied to the icon for each state
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

        this._buildIndicator();
        this._connectDbus();
        this._bindShortcut();
    }

    disable() {
        this._unbindShortcut();
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

        // ---- popup menu ----
        const menu = this._indicator.menu;

        // Status row
        this._statusItem = new PopupMenu.PopupMenuItem(_('Idle'), { reactive: false });
        menu.addMenuItem(this._statusItem);

        // Last transcript row (hidden when empty)
        this._transcriptItem = new PopupMenu.PopupMenuItem('', { reactive: false });
        this._transcriptItem.label.clutter_text.set_line_wrap(true);
        this._transcriptItem.visible = false;
        menu.addMenuItem(this._transcriptItem);

        menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Toggle action
        this._toggleItem = new PopupMenu.PopupMenuItem(_('Start Recording'));
        this._toggleItem.connect('activate', () => this._toggle());
        menu.addMenuItem(this._toggleItem);

        // Settings shortcut
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

        const iconName = STATE_ICONS[state] ?? STATE_ICONS['idle'];
        const styleClass = STATE_STYLE_CLASSES[state] ?? STATE_STYLE_CLASSES['idle'];

        // Update icon
        this._icon.icon_name = iconName;
        // Swap state style classes
        for (const cls of Object.values(STATE_STYLE_CLASSES))
            this._icon.remove_style_class_name(cls);
        this._icon.add_style_class_name(styleClass);

        // Update status label
        const labels = {
            idle:         _('Idle'),
            recording:    _('Recording…'),
            transcribing: _('Transcribing…'),
            error:        _('Error'),
        };
        this._statusItem.label.text = labels[state] ?? state;

        // Update transcript row
        if (state === 'idle' && this._lastTranscript) {
            const short = this._lastTranscript.length > 80
                ? this._lastTranscript.slice(0, 80) + '…'
                : this._lastTranscript;
            this._transcriptItem.label.text = short;
            this._transcriptItem.visible = true;
        } else if (state !== 'idle') {
            this._transcriptItem.visible = false;
        }

        // Toggle action label
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
                null,   // cancellable
                Gio.DBusProxyFlags.NONE,
            );

            // Listen for StateChanged signals
            this._signalId = this._proxy.connectSignal(
                'StateChanged',
                (_proxy, _sender, [state, transcript]) => {
                    this._onStateChanged(state, transcript);
                },
            );

            // Query initial status
            this._refreshStatus();

            // Watch for daemon appearing / disappearing
            this._nameWatchId = Gio.DBus.session.watch_name(
                DBUS_BUS_NAME,
                Gio.BusNameWatcherFlags.NONE,
                () => this._refreshStatus(),        // appeared
                () => this._updateIndicator('idle', ''), // vanished
            );

            // Auto-start the daemon if configured
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

        // Show notification when transcription completes
        if (state === 'idle' && transcript &&
            this._settings.get_boolean('show-transcript-notification')) {
            this._notify(_('Transcription complete'), transcript);
        }
        if (state === 'error') {
            this._notify(_('Voxput error'), _('Recording or transcription failed.'));
        }
    }

    _refreshStatus() {
        if (!this._proxy)
            return;
        try {
            // GetStatus returns (sss) struct: [state, transcript, error]
            const result = this._proxy.GetStatusSync();
            if (result && result[0]) {
                const [state, transcript, _error] = result[0];
                this._updateIndicator(state, transcript);
            }
        } catch (_e) {
            // daemon not running yet — stay idle
        }
    }

    // -----------------------------------------------------------------------
    // Actions
    // -----------------------------------------------------------------------

    _toggle() {
        if (!this._proxy) return;
        try {
            this._proxy.ToggleRemote((_result, error) => {
                if (error)
                    logError(error, 'Voxput: toggle failed');
            });
        } catch (e) {
            logError(e, 'Voxput: toggle error');
        }
    }

    _ensureDaemonRunning() {
        // Trigger D-Bus activation; voxputd will start if it is not running.
        try {
            this._proxy.GetStatusRemote((_r, _e) => {});
        } catch (_e) {}
    }

    // -----------------------------------------------------------------------
    // Keyboard shortcut
    // -----------------------------------------------------------------------

    _bindShortcut() {
        Main.wm.addKeybinding(
            'toggle-recording',
            this._settings,
            Meta.KeyBindingFlags.NONE,
            Shell.ActionMode.NORMAL | Shell.ActionMode.OVERVIEW,
            () => this._toggle(),
        );
    }

    _unbindShortcut() {
        Main.wm.removeKeybinding('toggle-recording');
    }

    // -----------------------------------------------------------------------
    // Notifications
    // -----------------------------------------------------------------------

    _notify(title, body) {
        // Reuse a single notification source for the extension.
        if (!this._notifSource) {
            this._notifSource = new MessageTray.Source({
                title: 'Voxput',
                iconName: 'audio-input-microphone-symbolic',
            });
            Main.messageTray.add(this._notifSource);
            this._notifSource.connect('destroy', () => {
                this._notifSource = null;
            });
        }

        const notification = new MessageTray.Notification({
            source: this._notifSource,
            title,
            body,
        });
        this._notifSource.addNotification(notification);
    }
}
