import de.undercouch.gradle.tasks.download.Download;
import java.time.Duration;
import java.time.Instant;

plugins {
    id("de.undercouch.download") version "[5.6.0, 6)"
}

tasks {
    val cargoFormat = register("cargoFormat", Exec::class) {
        commandLine("cargo", "fmt")
    }

    val cargoClippy = register("cargoClippy", Exec::class) {
        mustRunAfter(cargoFormat)

        commandLine("cargo", "clippy")
    }

    val cargoBuild = register("cargoBuild", Exec::class) {
        mustRunAfter(cargoClippy, cargoFormat)

        commandLine("cargo", "build")
    }

    val cargoFormatCheck = register("cargoFormatCheck", Exec::class) {
        mustRunAfter(cargoFormat)

        commandLine("cargo", "fmt", "--check")
    }

    val cargoClean = register("cargoClean", Exec::class) {
        commandLine("cargo", "clean")
    }

    val compile = register("compile") {
        dependsOn(cargoBuild)
    }

    val check = register("check") {
        dependsOn(cargoFormatCheck, cargoClippy)
    }

    val downloadCaseFoldingTxt = register("downloadCaseFoldingTxt", Download::class) {
        val outputFile = getTemporaryDir().toPath().resolve("CaseFolding.txt").toFile()

        src("https://www.unicode.org/Public/UCD/latest/ucd/CaseFolding.txt")
        dest(outputFile)
        overwrite(true)

        onlyIf {
            !outputFile.exists() ||
                outputFile.lastModified() < Instant.now()
                    .minus(Duration.ofDays(1))
                    .toEpochMilli()
        }
    }

    val regenerateUnicodeCasing = register("regenerateUnicodeCasing", Exec::class) {
        val caseFoldingTxt = downloadCaseFoldingTxt.map { it.dest }
        val outputFile = project.parent!!.layout.projectDirectory.file("src/encoding/unicode/case_folding.rs")

        inputs.file(caseFoldingTxt)

        commandLine(
            "cargo", "run", "--"
        )

        argumentProviders.add(CommandLineArgumentProvider {
            listOf(
                "--case-folding-file", caseFoldingTxt.get().toString(),
                "--output-file", outputFile.toString()
            )
        })


    }

    register("fmt") {
        dependsOn(cargoFormat)
    }

    register("build") {
        dependsOn(check, compile)
    }

    register("clean") {
        dependsOn(cargoClean)
    }
}