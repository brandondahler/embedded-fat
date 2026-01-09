
tasks {

    val cargoFormat by registering(Exec::class) {
        commandLine("cargo", "fmt")
    }

    val cargoClippy by registering(Exec::class) {
        shouldRunAfter(cargoFormat)

        commandLine("cargo", "clippy")
    }

    val cargoTest by registering(Exec::class) {
        shouldRunAfter(cargoClippy, cargoFormat)

        commandLine("cargo", "llvm-cov", "test", "--html", "--features", "std")
    }

    val cargoBuild by registering(Exec::class) {
        shouldRunAfter(cargoClippy, cargoFormat, cargoTest)

        commandLine("cargo", "build")
    }

    val cargoBuildRelease by registering(Exec::class) {
        shouldRunAfter(cargoClippy, cargoFormat, cargoBuild, cargoTest)

        commandLine("cargo", "build", "--release")
    }

    val cargoFormatCheck by registering(Exec::class) {
        shouldRunAfter(cargoFormat)

        commandLine("cargo", "fmt", "--check")
    }

    val compile by registering {
        dependsOn(cargoBuildRelease)
    }

    val check by registering {
        dependsOn(cargoFormatCheck, cargoClippy)
    }

    val test by registering {
        dependsOn(cargoTest)
    }

    register("fmt") {
        dependsOn(cargoFormat)
    }

    register("build") {
        dependsOn(check, compile, test)
    }
}