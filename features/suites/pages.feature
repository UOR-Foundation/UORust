Feature: pages

  @PG-01 @build
  Scenario: Pages identity preflights use exactly one mandatory unskippable guard in both workflows, refuse a stale or malformed repository, and require ten named and executed negative controls without failures, omissions or skips; an incomplete result exits nonzero with model-defined PAGES_IDENTITY_CONTROLS_INCOMPLETE
    Given the canonical source-pinned repository identity and model/pages-identity-diagnostics.toml
    When both workflow preflights and the strict ten-control result gate are exercised
    Then stale repository identities and disabled, ignored, missing or skipped controls are refused
    And the failure emits the model-defined PAGES_IDENTITY_CONTROLS_INCOMPLETE code with a nonzero exit
