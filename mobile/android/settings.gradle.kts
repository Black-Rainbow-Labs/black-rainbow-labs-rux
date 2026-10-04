pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "rux-mobile"
include(":app")
// The Kotlin half of the shell, which an application outside this repository includes by path.
include(":rux-shell")
