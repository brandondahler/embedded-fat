val featuresMatrix = listOf(
    listOf("async"),
    listOf("sync"),
    listOf("unicode-case-folding"),
    listOf("async", "sync"),
    listOf("async", "unicode-case-folding"),
    listOf("sync", "unicode-case-folding"),
    listOf("async", "sync", "unicode-case-folding"),
);

tasks {
    val cargoFormat by registering(Exec::class) {
        commandLine("cargo", "fmt")
    }

    val cargoFormatCheck by registering(Exec::class) {
        shouldRunAfter(cargoFormat)

        commandLine("cargo", "fmt", "--check")
    }

    val cargoClippy by registering(Exec::class) {
        shouldRunAfter(cargoFormat)

        commandLine("cargo", "clippy")
    }

    val cargoBuild by registering(Exec::class) {
        shouldRunAfter(cargoFormat, cargoFormatCheck, cargoClippy)

        commandLine("cargo", "build")
    }

    val cargoBuildFeaturesTasks: MutableList<TaskProvider<Exec>> = mutableListOf()

    featuresMatrix.forEach {
        val name = it.map { toTitleCase(it) }.joinToString("")

        val taskProvider = register("cargoBuildFeatures$name", Exec::class) {
            shouldRunAfter(cargoBuild)

            commandLine("cargo", "build", "--no-default-features", "--features", it.joinToString(","))
        }

        cargoBuildFeaturesTasks.add(taskProvider)
    }

    val cargoTest by registering(Exec::class) {
        shouldRunAfter(cargoClippy, cargoFormat, cargoBuild)

        commandLine(
            "cargo", "llvm-cov", "test",
            "--output-dir", "target/coverage/default",
            "--html")
    }

    val cargoTestWithoutUnicodeCasesFolding by registering(Exec::class) {
        shouldRunAfter(cargoClippy, cargoFormat, cargoTest, "cargoBuildFeaturesAsyncSync")

        commandLine(
            "cargo", "llvm-cov", "test",
            "--output-dir", "target/coverage/without-unicode-case-folding",
            "--html",
            "--no-default-features",
            "--features", "async,sync")
    }

    val cargoClean by registering(Exec::class) {
        commandLine("cargo", "clean")
    }

    val compile by registering {
        dependsOn(cargoBuild, cargoBuildFeaturesTasks)
    }

    val check by registering {
        dependsOn(cargoFormatCheck, cargoClippy)
    }

    val test by registering {
        dependsOn(cargoTest, cargoTestWithoutUnicodeCasesFolding)
    }

    val fmt by registering {
        dependsOn(cargoFormat)
    }

    register("build") {
        dependsOn(check, compile, test)
    }

    register("devBuild") {
        dependsOn(fmt, check, cargoBuild, cargoTest)
    }

    register("clean") {
        dependsOn(cargoClean)
    }
}

fun toTitleCase(value: String): String {
    return value.split('-')
        .joinToString("") { word ->
            word.replaceFirstChar {
                if (it.isLowerCase()) it.titlecase() else it.toString()
            }
        }
}