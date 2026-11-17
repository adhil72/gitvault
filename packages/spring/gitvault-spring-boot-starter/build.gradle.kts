description = "Spring Boot starter that loads a GitVault environment into the Spring Environment."

// Compiled against this Boot line; the starter uses only APIs that exist in every Boot 3.x release.
val bootVersion = "3.5.16"

dependencies {
    api(project(":gitvault"))
    // The application brings its own Spring Boot.
    compileOnly("org.springframework.boot:spring-boot:$bootVersion")

    testImplementation(platform("org.springframework.boot:spring-boot-dependencies:$bootVersion"))
    testImplementation("org.springframework.boot:spring-boot-starter-test")
    testImplementation(testFixtures(project(":gitvault")))
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}
