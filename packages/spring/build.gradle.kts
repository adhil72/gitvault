// Shared settings for both modules. Java 17 is the baseline of Spring Boot 3, and the core
// module has no dependencies so it also runs on any JVM app (Kotlin, Quarkus, plain Java).
subprojects {
    apply(plugin = "java-library")
    apply(plugin = "maven-publish")

    repositories {
        mavenCentral()
    }

    extensions.configure<JavaPluginExtension> {
        withSourcesJar()
        withJavadocJar()
    }

    tasks.withType<JavaCompile>().configureEach {
        options.release = 17
        options.encoding = "UTF-8"
        options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
    }

    tasks.withType<Javadoc>().configureEach {
        (options as StandardJavadocDocletOptions).addStringOption("Xdoclint:none", "-quiet")
    }

    tasks.withType<Test>().configureEach {
        useJUnitPlatform()
    }

    extensions.configure<PublishingExtension> {
        publications {
            create<MavenPublication>("maven") {
                from(components["java"])
                pom {
                    name = project.name
                    url = "https://github.com/adhil72/gitvault"
                    licenses {
                        license {
                            name = "MIT"
                            url = "https://opensource.org/licenses/MIT"
                        }
                    }
                    scm { url = "https://github.com/adhil72/gitvault" }
                }
            }
        }
    }
}
