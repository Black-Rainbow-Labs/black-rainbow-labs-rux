package com.blackrainbowlabs.rux

import android.os.Bundle
import android.view.View
import androidx.activity.OnBackPressedCallback
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import com.google.androidgamesdk.GameActivity
import com.google.androidgamesdk.gametextinput.State

/**
 * The activity every rux application on Android runs in.
 *
 * It is a `GameActivity`, not the framework's `NativeActivity`, for three reasons that each need Kotlin:
 * the soft keyboard (GameActivity owns an `InputConnection` and hands the text to Rust), AccessKit (eframe
 * only allows it with GameActivity), and the window insets (Android only reports them to a `View`).
 *
 * An application subclasses this in its own package so it can add its own Kotlin (permissions, services)
 * and names the subclass in its manifest. The Rust library is found through the `android.app.lib_name`
 * meta-data on that activity, exactly as GameActivity documents.
 */
open class RuxActivity : GameActivity() {

    /**
     * Draws edge to edge and starts listening for the back gesture.
     * @param savedInstanceState - the state Android saved, passed through
     */
    override fun onCreate(savedInstanceState: Bundle?) {
        // The application draws behind the system bars and keeps its own content clear of them, the way it
        // does on iOS. The insets that make that possible arrive in onApplyWindowInsets below.
        WindowCompat.setDecorFitsSystemWindows(window, false)
        super.onCreate(savedInstanceState)
        RuxBridge.attach(this)
        onBackPressedDispatcher.addCallback(this, backCallback)
    }

    /**
     * Records what the system bars, the cutout and the keyboard cover, in pixels, for Rust to read.
     * @param v - the view the insets were applied to
     * @param insets - the insets Android reports
     */
    override fun onApplyWindowInsets(v: View, insets: WindowInsetsCompat): WindowInsetsCompat {
        val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
        val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
        RuxBridge.setInsets(bars.top, bars.right, bars.bottom, bars.left, ime.bottom)
        return super.onApplyWindowInsets(v, insets)
    }

    /**
     * Records the keyboard's editor text for Rust, then lets GameActivity pass it to native code as usual.
     *
     * Rust reads the text from here rather than through `GameActivity_getTextInputState`, because that
     * call hands Rust a null pointer before the keyboard has typed anything and `android-activity` 0.6.1
     * builds a slice from it: a debug build panics on the precondition check, and a release build has
     * undefined behaviour.
     * @param state - the editor state the keyboard just produced
     * @param dismissed - whether the keyboard was closed
     */
    override fun stateChanged(state: State, dismissed: Boolean) {
        RuxBridge.keyboardChanged(state.text ?: "")
        super.stateChanged(state, dismissed)
    }

    /**
     * The keyboard's action key (Done, Send, Go). winit drops these, so they are counted here for Rust.
     * @param action - the `EditorInfo.IME_ACTION_*` value
     */
    override fun onEditorAction(action: Int) {
        RuxBridge.editorAction()
        super.onEditorAction(action)
    }

    /** Empties the keyboard's editor, on the UI thread, so the next field starts from nothing. */
    internal fun clearKeyboard() {
        setTextInputState(State("", 0, 0, -1, -1))
    }

    /** Whether Rust has somewhere to go back to. Set through `RuxBridge.setHandlesBack`. */
    private var rustHandlesBack = false

    /**
     * The back gesture goes to Rust while the application says it has somewhere to go back to. At the
     * root it moves the task to the back instead of finishing the activity.
     *
     * Finishing is Android's default and it is wrong for a rux application: the Rust side (its threads,
     * its sockets, its statics) lives as long as the process, and android-activity starts a second
     * `android_main` beside the first when a destroyed activity is opened again in the same process.
     * Measured on the emulator: after a back gesture finished the activity, opening the app again left
     * the splash screen up with nothing drawing. Moving the task to the back keeps the one activity and
     * the one `android_main`, which is also what an assistant that listens in the background wants.
     */
    private val backCallback = object : OnBackPressedCallback(true) {
        override fun handleOnBackPressed() {
            if (rustHandlesBack) RuxBridge.backPressed() else moveTaskToBack(true)
        }
    }

    /**
     * Records whether Rust wants the next back gesture. Called on the UI thread by RuxBridge.
     * @param handles - whether the application wants the next back gesture
     */
    internal fun setBackEnabled(handles: Boolean) {
        rustHandlesBack = handles
    }
}
