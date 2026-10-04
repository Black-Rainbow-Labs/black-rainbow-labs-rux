// The Kotlin half of every rux application on Android: `RuxActivity` and `RuxBridge`.
//
// It is an Android library module so that an application outside this repository can include it by path
// from its own `settings.gradle.kts`:
//
//     include(":rux-shell")
//     project(":rux-shell").projectDir = file("C:/jason/dev/black-rainbow-labs-rux/mobile/android/rux-shell")
//
// and subclass `RuxActivity` in its own package. `mobile/android/app` is the example that does exactly that.
plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.blackrainbowlabs.rux.shell"
    compileSdk = 36

    defaultConfig {
        minSdk = 28
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    lint {
        abortOnError = false
    }
}

kotlin {
    jvmToolchain(17)
}

dependencies {
    // `android-activity` 0.6 is written against GameActivity 4.4.0 and says so in its README. The prefab
    // (C++ glue) half is deliberately not enabled: `android-activity` compiles its own copy of that glue,
    // and linking the upstream one as well gives two definitions of every GameActivity callback.
    api("androidx.games:games-activity:4.4.0")
    api("androidx.appcompat:appcompat:1.7.0")
    api("androidx.core:core-ktx:1.15.0")
    api("androidx.activity:activity-ktx:1.10.1")
}
