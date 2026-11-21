Feature: ci

  @CI-01 @build
  Scenario: PR impact reporting evaluates exact base/head Git changes using a trusted-base classifier and factory input graph; passive documentation is distinguished from model inputs, while missing, renamed or nonregular evidence never authorizes skipping native acceptance
    Given the fixture CI-01 needs
    When the suite exercises CI-01
    Then PR impact reporting evaluates exact base/head Git changes using a trusted-base classifier and factory input graph; passive documentation is distinguished from model inputs, while missing, renamed or nonregular evidence never authorizes skipping native acceptance

  @CI-02 @build
  Scenario: Trusted-base PR qualification planning resolves the normative just vv recipe and factory dependency graph to a deterministic per-obligation selection, expanding dependent checks, refusing unknown or policy changes with full execution, and never granting merge acceptance from an advisory plan
    Given the trusted-base factory graph and exact base/head Git revision comparison
    When the suite exercises CI-02
    Then the selected just vv constituent obligations match the factory input closure
    And an unknown, unsafe or changed trust policy routes to the full set
    And the advisory selection never claims merge qualification
