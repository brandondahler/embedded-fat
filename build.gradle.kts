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
    val cargoFormat = register("cargoFormat", Exec::class) {
        commandLine("cargo", "fmt")
    }

    val cargoFormatCheck = register("cargoFormatCheck", Exec::class) {
        mustRunAfter(cargoFormat)

        commandLine("cargo", "fmt", "--check")
    }

    val cargoClippy = register("cargoClippy", Exec::class) {
        mustRunAfter(cargoFormat)

        commandLine("cargo", "clippy")
    }

    val cargoBuild = register("cargoBuild", Exec::class) {
        mustRunAfter(cargoFormat, cargoFormatCheck, cargoClippy)

        commandLine("cargo", "build")
    }

    val cargoBuildFeaturesTasks: MutableList<TaskProvider<Exec>> = mutableListOf()

    featuresMatrix.forEach {
        val name = it.map { toTitleCase(it) }.joinToString("")

        val taskProvider = register("cargoBuildFeatures$name", Exec::class) {
            mustRunAfter(cargoBuild)

            commandLine("cargo", "build", "--no-default-features", "--features", it.joinToString(","))
        }

        cargoBuildFeaturesTasks.add(taskProvider)
    }

    val cargoTest = register("cargoTest", Exec::class) {
        mustRunAfter(cargoClippy, cargoFormat, cargoBuild)

        commandLine(
            "cargo", "llvm-cov", "test",
            "--output-dir", "target/coverage/default",
            "--html")
    }

    val cargoTestWithoutUnicodeCasesFolding = register("cargoTestWithoutUnicodeCasesFolding", Exec::class) {
        mustRunAfter(cargoClippy, cargoFormat, cargoTest)
        shouldRunAfter("cargoBuildFeaturesAsyncSync")

        commandLine(
            "cargo", "llvm-cov", "test",
            "--output-dir", "target/coverage/without-unicode-case-folding",
            "--html",
            "--no-default-features",
            "--features", "async,sync")
    }

    val cargoClean = register("cargoClean", Exec::class) {
        commandLine("cargo", "clean")
    }

    val compile = register("compile") {
        dependsOn(cargoBuild, cargoBuildFeaturesTasks)
    }

    val check = register("check") {
        dependsOn(cargoFormatCheck, cargoClippy)
    }

    val test = register("test") {
        dependsOn(cargoTest, cargoTestWithoutUnicodeCasesFolding)
    }

    val fmt = register("fmt") {
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