Feature: Optimizing a CosmWasm workspace

  Scenario: Optimizes workspace for the first time
    Given the user is in the workspace "cw-plus"
    When the user successfully runs cw-optimizoor for the first time
    Then 8 contracts are optimized
    And 8 wasm files exist in the artifacts dir
    And each artifact contains a function named "execute"

  Scenario: Reoptimizes workspace a second time
    Given the user is in the workspace "cw-plus"
    When the user successfully runs cw-optimizoor again
    Then 8 contracts are unchanged and skipped
    But 8 wasm files exist in the artifacts dir
    And each artifact contains a function named "execute"

  Scenario: Reoptimizes only changed contracts
    Given the user is in the workspace "cw-plus"
    And the user makes a change in the "cw1-subkeys" contract
    When the user successfully runs cw-optimizoor again
    Then 7 contracts are unchanged and skipped
    But "cw1_subkeys" is reoptimized
    And 8 wasm files exist in the artifacts dir
    And each artifact contains a function named "execute"

  Scenario: Reoptimizes deleted artifact
    Given the user is in the workspace "cw-plus"
    And the user deletes the artifact "cw1_subkeys"
    When the user successfully runs cw-optimizoor again
    Then 7 contracts are unchanged and skipped
    But "cw1_subkeys" is reoptimized
    And 8 wasm files exist in the artifacts dir
    And each artifact contains a function named "execute"
