plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "org.aiparentalcontrol.child"
    compileSdk = 35

    defaultConfig {
        applicationId = "org.aiparentalcontrol.child"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    // Two builds from one source. See README. Both are overt; the difference is
    // only how much text the AI layer can see, which is gated by store policy.
    flavorDimensions += "distribution"
    productFlavors {
        create("store") {
            dimension = "distribution"
            // Play-compliant: DNS filtering, usage limits, image AI, notification
            // previews. No AccessibilityService for monitoring, no READ_SMS.
        }
        create("sideload") {
            dimension = "distribution"
            applicationIdSuffix = ".deep"
            versionNameSuffix = "-deep"
            // Adds AccessibilityService full-text and optional SMS (declared in
            // src/sideload/AndroidManifest.xml). Ships its own updater.
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("androidx.lifecycle:lifecycle-service:2.8.4")
    implementation("com.squareup.okhttp3:okhttp:4.12.0")

    // The shared Rust core (packages/ffi) is generated into apc_ffi Kotlin
    // bindings and the native lib is bundled as a jniLibs .so. UniFFI uses JNA.
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    // implementation(project(":apc-core")) // module wrapping the generated bindings + .so

    testImplementation("junit:junit:4.13.2")
}
