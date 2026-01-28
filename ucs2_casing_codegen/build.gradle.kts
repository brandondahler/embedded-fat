import de.undercouch.gradle.tasks.download.Download;
import java.time.Duration;
import java.time.Instant;

plugins {
    id("de.undercouch.download") version "[5.6.0, 6)"
}

tasks {
    val cargoFormat by registering(Exec::class) {
        commandLine("cargo", "fmt")
    }

    val cargoClippy by registering(Exec::class) {
        shouldRunAfter(cargoFormat)

        commandLine("cargo", "clippy")
    }

    val cargoBuild by registering(Exec::class) {
        shouldRunAfter(cargoClippy, cargoFormat)

        commandLine("cargo", "build")
    }

    val cargoFormatCheck by registering(Exec::class) {
        shouldRunAfter(cargoFormat)

        commandLine("cargo", "fmt", "--check")
    }

    val cargoClean by registering(Exec::class) {
        commandLine("cargo", "clean")
    }

    val compile by registering {
        dependsOn(cargoBuild)
    }

    val check by registering {
        dependsOn(cargoFormatCheck, cargoClippy)
    }

    val regenerateUcs2Casing by registering(Exec::class) {
        val caseFoldingFileLocation = getTemporaryDir().toPath().resolve("CaseFolding.txt").toFile()
        val outputFile = project.parent!!.layout.projectDirectory.file("src/encoding/ucs2_character/case_folding.rs")

        commandLine(
            "cargo", "run", "--",
            "--case-folding-file", caseFoldingFileLocation,
            "--output-file", outputFile
        )

        doFirst {
            val yesterday = Instant.now().minus(Duration.ofDays(1))
            val shouldOverwrite = caseFoldingFileLocation.lastModified() < yesterday.toEpochMilli()

            download.run {
                src("https://www.unicode.org/Public/UCD/latest/ucd/CaseFolding.txt")
                dest(caseFoldingFileLocation)
                overwrite(shouldOverwrite)
            }
        }
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