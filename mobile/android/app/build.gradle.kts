plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.blackrainbowlabs.rux"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.blackrainbowlabs.rux"
        // `android-activity` 0.6 needs 21, and wgpu's Vulkan backend wants a device that really has
        // Vulkan 1.1 — 28 is the floor that gives both without carrying a fallback path nobody tests.
        minSdk = 28
        // **36.** Android enforces edge to edge from 35, and that used to be a reason to stay on 34: a
        // `NativeActivity` had no way to learn what the system bars covered. `RuxActivity` now reads the
        // real insets in `onApplyWindowInsets` and `rux-android` hands them to `rux`, so the application
        // draws behind the bars and keeps its content clear of them, the way it does on iOS.
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
        ndk {
            // `arm64-v8a` is every phone worth shipping to. `x86_64` is here because **the emulator on
            // this machine is x86_64** — `Medium_Phone_API_36.0` reports `sdk_gphone64_x86_64` — and an
            // arm64-only APK installs on it and then runs through binary translation or not at all.
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    // The Rust library is built by `build-android.ps1` and dropped here, rather than by a Gradle task
    // that shells out to cargo. Two reasons: the same script has to work with no Gradle at all for the
    // iOS side of this folder, and a cargo build inside a Gradle task hides cargo's own output.
    sourceSets {
        getByName("main") {
            jniLibs.srcDir("src/main/jniLibs")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
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
    implementation(project(":rux-shell"))
}
