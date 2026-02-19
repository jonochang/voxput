// Voxput extension preferences
// GNOME Shell 45+ (ESM module)

import Adw from 'gi://Adw';
import Gdk from 'gi://Gdk';
import Gio from 'gi://Gio';
import Gtk from 'gi://Gtk';

import { ExtensionPreferences, gettext as _ } from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

export default class VoxputPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        const settings = this.getSettings();

        // ---- Page ----
        const page = new Adw.PreferencesPage({
            title: _('General'),
            icon_name: 'preferences-system-symbolic',
        });
        window.add(page);

        // ---- Shortcut group ----
        const shortcutGroup = new Adw.PreferencesGroup({
            title: _('Keyboard Shortcut'),
            description: _('Shortcut to toggle recording. First press starts; second press stops and transcribes.'),
        });
        page.add(shortcutGroup);

        const shortcutRow = new Adw.ActionRow({
            title: _('Toggle Recording'),
            subtitle: _('Click to change'),
        });
        shortcutGroup.add(shortcutRow);

        // Show current shortcut as a label
        const shortcutLabel = new Gtk.ShortcutLabel({
            valign: Gtk.Align.CENTER,
        });
        const updateShortcutLabel = () => {
            const val = settings.get_strv('toggle-recording');
            shortcutLabel.accelerator = val.length > 0 ? val[0] : '';
        };
        updateShortcutLabel();
        settings.connect('changed::toggle-recording', updateShortcutLabel);
        shortcutRow.add_suffix(shortcutLabel);

        // Button to edit the shortcut
        const editButton = new Gtk.Button({
            label: _('Change'),
            valign: Gtk.Align.CENTER,
        });
        editButton.connect('clicked', () => {
            const dialog = new ShortcutDialog(settings, window);
            dialog.present();
        });
        shortcutRow.add_suffix(editButton);
        shortcutRow.activatable_widget = editButton;

        // ---- Notification group ----
        const notifGroup = new Adw.PreferencesGroup({
            title: _('Notifications'),
        });
        page.add(notifGroup);

        const notifRow = new Adw.SwitchRow({
            title: _('Show transcript notification'),
            subtitle: _('Display a GNOME notification when transcription completes.'),
        });
        settings.bind(
            'show-transcript-notification',
            notifRow,
            'active',
            Gio.SettingsBindFlags.DEFAULT,
        );
        notifGroup.add(notifRow);

        // ---- Daemon group ----
        const daemonGroup = new Adw.PreferencesGroup({
            title: _('Daemon'),
        });
        page.add(daemonGroup);

        const autoStartRow = new Adw.SwitchRow({
            title: _('Auto-start voxputd'),
            subtitle: _('Start the voxputd daemon automatically when the extension enables.'),
        });
        settings.bind(
            'daemon-auto-start',
            autoStartRow,
            'active',
            Gio.SettingsBindFlags.DEFAULT,
        );
        daemonGroup.add(autoStartRow);
    }
}

// ---------------------------------------------------------------------------
// Simple shortcut capture dialog
// ---------------------------------------------------------------------------

class ShortcutDialog extends Gtk.Dialog {
    constructor(settings, parent) {
        super({
            title: _('Set Shortcut'),
            transient_for: parent,
            modal: true,
            use_header_bar: 1,
        });

        this._settings = settings;

        const label = new Gtk.Label({
            label: _('Press the desired key combination, then release.'),
            margin_top: 20,
            margin_bottom: 20,
            margin_start: 20,
            margin_end: 20,
        });
        this.get_content_area().append(label);

        const controller = new Gtk.EventControllerKey();
        controller.connect('key-pressed', (_ctrl, keyval, keycode, state) => {
            // Ignore bare modifier presses
            if ([
                Gdk.KEY_Control_L, Gdk.KEY_Control_R,
                Gdk.KEY_Shift_L, Gdk.KEY_Shift_R,
                Gdk.KEY_Alt_L, Gdk.KEY_Alt_R,
                Gdk.KEY_Super_L, Gdk.KEY_Super_R,
            ].includes(keyval)) return Gdk.EVENT_PROPAGATE;

            const mask = state & Gtk.accelerator_get_default_mod_mask();
            const accel = Gtk.accelerator_name_with_keycode(null, keyval, keycode, mask);
            if (accel) {
                settings.set_strv('toggle-recording', [accel]);
                this.close();
            }
            return Gdk.EVENT_STOP;
        });
        this.add_controller(controller);
    }
}
