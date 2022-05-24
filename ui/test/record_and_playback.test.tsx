import React from "react";
import { act, fireEvent, render, within } from "@testing-library/react";
import diff from "snapshot-diff";

import UiMain from "../src/ui_main";

beforeAll(() => {
  jest.spyOn(Date.prototype, "toDateString").mockReturnValue("Mocked Date");
  jest
    .spyOn(Date.prototype, "toLocaleTimeString")
    .mockReturnValue("Mocked Date");

  document.body.style.width = "1024px";
  document.body.style.height = "768px";
});

describe("app [integration]", () => {
  it("can record and playback a clip", async () => {
    const app = render(<UiMain inMemory={true} />);
    const startRecording = app.getByRole("button", {
      name: "Start Recording",
    });
    fireEvent.click(startRecording);

    let nextFragment = app.asFragment();
    expect(nextFragment).toMatchSnapshot("1 initial");
    let prevFragment = nextFragment;

    const stopRecording = await app.findByRole("button", {
      name: "Complete Recording",
    });
    nextFragment = app.asFragment();
    expect(diff(prevFragment, nextFragment)).toMatchSnapshot("1 recording");
    prevFragment = nextFragment;

    await act(async () => {
      await new Promise((res) => setTimeout(res, 200));
      fireEvent.click(stopRecording);
      await new Promise((res) => setTimeout(res, 200));
    });

    const play = await app.findByRole("button", { name: "Play" });
    // Rename the clip since it has a date in it.
    const clipName = app.getByTestId("current-clip-name") as HTMLInputElement;
    expect(clipName.value).toMatch(/20\d\d-\d\d-\d\d .*/);
    fireEvent.focus(clipName);
    fireEvent.change(clipName, { target: { value: "New clip name" } });
    fireEvent.blur(clipName);

    const dismissRename = within(
      app.getByText(/Renamed.*to.*/).parentElement
    ).getByRole("button", { name: "Dismiss" });
    fireEvent.click(dismissRename);

    await app.findByDisplayValue("New clip name");

    nextFragment = app.asFragment();
    expect(diff(prevFragment, nextFragment)).toMatchSnapshot(
      "2 done recording"
    );
    prevFragment = nextFragment;

    fireEvent.click(play);

    const pause = await app.findByRole("button", { name: "Pause" });
    nextFragment = app.asFragment();
    expect(diff(prevFragment, nextFragment)).toMatchSnapshot("3 playing");
    prevFragment = nextFragment;

    fireEvent.click(pause);

    await app.findByRole("button", { name: "Play" });
    nextFragment = app.asFragment();
    expect(diff(prevFragment, nextFragment)).toMatchSnapshot("4 paused");
    prevFragment = nextFragment;
  });
});
