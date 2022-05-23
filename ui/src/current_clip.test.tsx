import React from "react";
import { render, fireEvent } from "@testing-library/react";

import CurrentClip from "./current_clip";

describe("CurrentClip", () => {
  it("renders playing state and can be stopped", () => {
    const handlePlay = jest.fn();
    const handleStop = jest.fn();
    const handleRename = jest.fn();
    const handleDelete = jest.fn();
    const handleDrawCurrentClipWaveform = jest.fn((width, height) => {
      return Buffer.from(Array(width * height * 4).fill(0));
    });

    const currentClip = render(
      <CurrentClip
        clip={{
          date: new Date("2022-05-20T19:34:29.074Z"),
          id: 1n,
          name: "Current clip",
        }}
        time={125}
        timePercent={0.2}
        streaming={true}
        onPlay={handlePlay}
        onStop={handleStop}
        onRename={handleRename}
        onDelete={handleDelete}
        drawCurrentClipWaveform={handleDrawCurrentClipWaveform}
      />
    );

    expect(handleDrawCurrentClipWaveform).toHaveBeenCalledTimes(1);

    const stop = currentClip.getByTestId("current-clip-toggle-playback");

    expect(stop.textContent).toEqual("Pause");
    fireEvent.click(stop);
    expect(handleStop).toHaveBeenCalledTimes(1);
    expect(currentClip.getByTestId("current-clip-time").textContent).toEqual(
      "125.00"
    );
  });
  it("renders stopped state and can be played", () => {
    const handlePlay = jest.fn();
    const handleStop = jest.fn();
    const handleRename = jest.fn();
    const handleDelete = jest.fn();
    const handleDrawCurrentClipWaveform = jest.fn((width, height) => {
      return Buffer.from(Array(width * height * 4).fill(0));
    });

    const currentClip = render(
      <CurrentClip
        clip={{
          date: new Date("2022-05-20T19:34:29.074Z"),
          id: 1n,
          name: "Current clip",
        }}
        time={125}
        timePercent={0.2}
        streaming={false}
        onPlay={handlePlay}
        onStop={handleStop}
        onRename={handleRename}
        onDelete={handleDelete}
        drawCurrentClipWaveform={handleDrawCurrentClipWaveform}
      />
    );

    expect(handleDrawCurrentClipWaveform).toHaveBeenCalledTimes(1);

    const play = currentClip.getByTestId("current-clip-toggle-playback");

    expect(play.textContent).toEqual("Play");
    fireEvent.click(play);
    expect(handlePlay).toHaveBeenCalledTimes(1);
    expect(currentClip.getByTestId("current-clip-time").textContent).toEqual(
      "125.00"
    );
  });
  it("can be deleted", () => {
    const handlePlay = jest.fn();
    const handleStop = jest.fn();
    const handleRename = jest.fn();
    const handleDelete = jest.fn();
    const handleDrawCurrentClipWaveform = jest.fn((width, height) => {
      return Buffer.from(Array(width * height * 4).fill(0));
    });

    const currentClip = render(
      <CurrentClip
        clip={{
          date: new Date("2022-05-20T19:34:29.074Z"),
          id: 1n,
          name: "Current clip",
        }}
        time={125}
        timePercent={0.2}
        streaming={false}
        onPlay={handlePlay}
        onStop={handleStop}
        onRename={handleRename}
        onDelete={handleDelete}
        drawCurrentClipWaveform={handleDrawCurrentClipWaveform}
      />
    );

    expect(handleDrawCurrentClipWaveform).toHaveBeenCalledTimes(1);

    const deleteBtn = currentClip.getByTestId("current-clip-delete");

    expect(deleteBtn.title).toEqual("Delete this clip");
    fireEvent.click(deleteBtn);
    expect(handleDelete).toHaveBeenCalledTimes(1);
  });
  it("can be renamed", () => {
    const handlePlay = jest.fn();
    const handleStop = jest.fn();
    const handleRename = jest.fn();
    const handleDelete = jest.fn();
    const handleDrawCurrentClipWaveform = jest.fn((width, height) => {
      return Buffer.from(Array(width * height * 4).fill(0));
    });

    const currentClip = render(
      <CurrentClip
        clip={{
          date: new Date("2022-05-20T19:34:29.074Z"),
          id: 1n,
          name: "Current clip",
        }}
        time={125}
        timePercent={0.2}
        streaming={false}
        onPlay={handlePlay}
        onStop={handleStop}
        onRename={handleRename}
        onDelete={handleDelete}
        drawCurrentClipWaveform={handleDrawCurrentClipWaveform}
      />
    );

    expect(handleDrawCurrentClipWaveform).toHaveBeenCalledTimes(1);

    const clipName = currentClip.getByTestId("current-clip-name");
    fireEvent.focus(clipName);
    fireEvent.change(clipName, { target: { value: "New clip name" } });
    fireEvent.blur(clipName);

    expect(handleRename).toHaveBeenCalledWith("New clip name");
  });
});
