plugins {
    `java-test-fixtures`
}

description = "Load GitVault secrets from the gitvault CLI on the JVM. No dependencies."

dependencies {
    testImplementation(platform("org.junit:junit-bom:5.13.4"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}
