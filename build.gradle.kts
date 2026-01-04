
tasks {
    val cargoBuild by registering(Exec::class) {
        commandLine("cargo", "build")
    }

    val cargoBuildRelease by registering(Exec::class) {
        commandLine("cargo", "build", "--release")
    }

    val cargoClippy by registering(Exec::class) {
        commandLine("cargo", "clippy")
    }

    val cargoFormat by registering(Exec::class) {
        commandLine("cargo", "fmt")
    }

    val cargoFormatCheck by registering(Exec::class) {
        commandLine("cargo", "fmt", "--check")
    }

    val cargoTest by registering(Exec::class) {
        commandLine("cargo", "llvm-cov", "test", "--html", "--features", "std")
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
        dependsOn("cargoFormat")
    }

    register("build") {
        dependsOn(check, compile, test)
    }
}