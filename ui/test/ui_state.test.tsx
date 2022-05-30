import { UiState } from "oxygen-core";

describe("UiState", () => {
  it("can be created twice without crashing", () => {
    new UiState(
      () => {},
      () => {},
      true
    );
    new UiState(
      () => {},
      () => {},
      true
    );
  });
});
