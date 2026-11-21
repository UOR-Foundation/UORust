Feature: ci

  @CI-01 @build
  Scenario: PR impact reporting evaluates exact base/head Git changes using a trusted-base classifier and factory input graph; passive documentation is distinguished from model inputs, while missing, renamed or nonregular evidence never authorizes skipping native acceptance
    Given the fixture CI-01 needs
    When the suite exercises CI-01
    Then PR impact reporting evaluates exact base/head Git changes using a trusted-base classifier and factory input graph; passive documentation is distinguished from model inputs, while missing, renamed or nonregular evidence never authorizes skipping native acceptance
