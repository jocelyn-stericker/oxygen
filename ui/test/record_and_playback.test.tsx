import React from "react";
import { act, fireEvent, render, within } from "@testing-library/react";
import diff from "snapshot-diff";
import postcss from "postcss";
import tailwind from "tailwindcss";

import UiMain from "../src/ui_main";

beforeAll(async () => {
  jest
    .spyOn(Date.prototype, "toLocaleDateString")
    .mockReturnValue("Mocked Date");
  jest
    .spyOn(Date.prototype, "toLocaleTimeString")
    .mockReturnValue("Mocked Date");

  const styleSheet = document.createElement("style");
  styleSheet.innerText = (
    await postcss([tailwind("tailwind.config.js")]).process(
      " @tailwind base; @tailwind components; @tailwind utilities;",
      {
        from: "../src/index.css",
      }
    )
  ).css;
  document.head.appendChild(styleSheet);
});

describe("app [integration]", () => {
  it("can record and playback a clip", async () => {
    const app = render(
      <div style={{ width: 1024, height: 768 }}>
        <UiMain inMemory={true} />
      </div>
    );
    const startRecording = app.getByRole("button", {
      name: "Start Recording",
    });
    fireEvent.click(startRecording);

    let nextFragment = app.asFragment();
    const initialFragment = nextFragment;
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
    await app.findByRole("button", { name: "Pause" });

    fireEvent.click(pause);

    fireEvent.click(
      app.getByRole("tab", { name: "Record New Clip", selected: false })
    );
    await app.findByRole("tab", { name: "Record New Clip", selected: true });

    nextFragment = app.asFragment();
    expect(diff(initialFragment, nextFragment)).toMatchSnapshot(
      "3 delta from initial"
    );
  });
});
