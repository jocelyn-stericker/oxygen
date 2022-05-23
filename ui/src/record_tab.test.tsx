import React from "react";
import { render, fireEvent } from "@testing-library/react";

import RecordTab from "./record_tab";

describe("RecordTab", () => {
  it("renders playing state and can be completed", () => {
    const handleRecord = jest.fn();
    const handleStop = jest.fn();

    const recordTab = render(
      <RecordTab streaming={true} onRecord={handleRecord} onStop={handleStop} />
    );

    const toggle = recordTab.getByTestId("toggle-record");
    fireEvent.click(toggle);

    expect(toggle.textContent).toEqual("Complete Recording");
    expect(handleStop).toHaveBeenCalledTimes(1);
    expect(handleRecord).toHaveBeenCalledTimes(0);
  });

  it("renders stopped state and can be started", () => {
    const handleRecord = jest.fn();
    const handleStop = jest.fn();

    const recordTab = render(
      <RecordTab
        streaming={false}
        onRecord={handleRecord}
        onStop={handleStop}
      />
    );

    const toggle = recordTab.getByTestId("toggle-record");
    fireEvent.click(toggle);

    expect(toggle.textContent).toEqual("Start Recording");
    expect(handleStop).toHaveBeenCalledTimes(0);
    expect(handleRecord).toHaveBeenCalledTimes(1);
  });
});
