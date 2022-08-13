Feature: Optimizing a CosmWasm workspace

  Scenario: Optimizes workspace for the first time
    Given the user is in the workspace "cw-plus"
    When the user runs cw-optimizoor for the first time
    Then 10 wasm files exist in the artifacts dir
    And each artifact contains a function named "execute"

  Scenario: Reoptimizes workspace a second time
    Given the user is in the workspace "cw-plus"
    When the user runs cw-optimizoor again
    Then 10 wasm files exist in the artifacts dir
    And each artifact contains a function named "execute"
