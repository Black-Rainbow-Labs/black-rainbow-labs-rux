package com.blackrainbowlabs.rux

import android.os.Handler
import android.os.Looper
import java.lang.ref.WeakReference

/**
 * The values the Kotlin side knows and the Rust side needs, and the one call back into Rust.
 *
 * Rust reads these through JNI once a frame (`rux-android` does it), so nothing here has to know when the
 * Rust thread is ready. The single exception is [nativeWake]: an idle egui application only draws when
 * something asks it to, and a change that happens in Kotlin (the keyboard opening, the back gesture) has
 * to be able to ask. Rust registers that method with `RegisterNatives` when it starts, and until it has,
 * [wake] does nothing.
 */
object RuxBridge {
    private var activity: WeakReference<RuxActivity> = WeakReference(null)
    private val mainThread = Handler(Looper.getMainLooper())

    /** Top, right, bottom, left of the system bars and cutout, then the keyboard's height. Pixels. */
    @Volatile private var insets: IntArray = IntArray(5)

    /** The text in the keyboard's editor, as the keyboard last left it. */
    @Volatile private var keyboardText: String = ""

    /** How many keyboard action key presses Rust has not read yet. */
    @Volatile private var editorActions: Int = 0

    /** How many back gestures Rust has not read yet. */
    @Volatile private var backPresses: Int = 0

    /** Set by Rust once [nativeWake] is registered. */
    @JvmStatic @Volatile var nativeReady: Boolean = false

    /**
     * Remembers the activity so later calls can reach it.
     * @param owner - the activity that just started
     */
    internal fun attach(owner: RuxActivity) {
        activity = WeakReference(owner)
    }

    /**
     * Stores the latest insets and asks Rust to draw a frame with them.
     * @param top - status bar or cutout, pixels
     * @param right - right edge, pixels
     * @param bottom - navigation bar, pixels
     * @param left - left edge, pixels
     * @param ime - the soft keyboard's height, pixels, 0 when it is closed
     */
    internal fun setInsets(top: Int, right: Int, bottom: Int, left: Int, ime: Int) {
        insets = intArrayOf(top, right, bottom, left, ime)
        wake()
    }

    /**
     * Stores the keyboard's editor text and wakes Rust to read it.
     * @param text - the whole editor text
     */
    internal fun keyboardChanged(text: String) {
        keyboardText = text
        wake()
    }

    /** Counts a press of the keyboard's action key and wakes Rust. */
    internal fun editorAction() {
        editorActions += 1
        wake()
    }

    /** The keyboard's editor text. Read by Rust once a frame while a text field has focus. */
    @JvmStatic fun keyboardText(): String = keyboardText

    /** The action key presses since the last call, and resets the count. Read by Rust. */
    @JvmStatic fun takeEditorActions(): Int {
        val count = editorActions
        editorActions = 0
        return count
    }

    /**
     * Empties the keyboard's editor. Called by Rust when a text field gains focus, so the keyboard never
     * holds text from a field that is gone. The stored copy is emptied at once and the editor on the UI
     * thread.
     */
    @JvmStatic fun clearKeyboard() {
        keyboardText = ""
        mainThread.post { activity.get()?.clearKeyboard() }
    }

    /** Counts a back gesture for Rust to read, and wakes Rust to read it. */
    internal fun backPressed() {
        backPresses += 1
        wake()
    }

    /** The latest insets: top, right, bottom, left, keyboard. Pixels. Read by Rust. */
    @JvmStatic fun insets(): IntArray = insets.copyOf()

    /** The back gestures since the last call, and resets the count. Read by Rust. */
    @JvmStatic fun takeBackPresses(): Int {
        val count = backPresses
        backPresses = 0
        return count
    }

    /**
     * Whether the next back gesture goes to Rust. Called by Rust from its own thread, so the change is
     * posted to the UI thread, which is the only thread the dispatcher may be touched on.
     * @param handles - true while the application has a screen to go back from
     */
    @JvmStatic fun setHandlesBack(handles: Boolean) {
        mainThread.post { activity.get()?.setBackEnabled(handles) }
    }

    /** Asks Rust for a frame, once Rust is listening. */
    @JvmStatic fun wake() {
        if (nativeReady) nativeWake()
    }

    /** Implemented in Rust and registered at startup. Requests a repaint of the egui context. */
    @JvmStatic external fun nativeWake()
}
