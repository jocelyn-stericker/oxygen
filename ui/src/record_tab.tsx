import React from "react";
import cx from "classnames";
import { RenderMode } from "oxygen-core";

import { Record, Stop } from "./icons";
import AudioView from "./audio_view";

export default function RecordTab({
  drawCurrentClip,
  streaming,
  onRecord,
  onStop,
  onSetRenderMode,
  renderMode,
  timeStart,
  timeEnd,
}: {
  drawCurrentClip: (width: number, height: number) => Buffer | null;
  streaming: boolean;
  onRecord: () => void;
  onStop: () => void;
  onSetRenderMode: (renderMode: RenderMode) => void;
  renderMode: RenderMode;
  timeStart: number;
  timeEnd: number;
}) {
  return (
    <div className="flex flex-col flex-grow overflow-hidden">
      <AudioView
        drawCurrentClip={drawCurrentClip}
        time={timeEnd}
        streaming={streaming}
        onSeek={() => {
          console.warn("TODO: implement seek in record tab?");
        }}
        onSetRenderMode={onSetRenderMode}
        renderMode={renderMode}
        timeStart={timeStart}
        timeEnd={timeEnd}
      />
      <div className="flex flex-row mb-4">
        <div className="flex-grow" />
        <button
          data-testid="toggle-record"
          className={cx(
            "px-4 rounded-md m-auto text-lg flex border-2 h-16 items-center",
            streaming
              ? "bg-white border-purple-900 text-purple-900 hover:bg-purple-100"
              : "bg-purple-900 text-white hover:bg-purple-800"
          )}
          onClick={(ev) => {
            ev.preventDefault();
            if (!streaming) {
              onRecord();
            } else {
              onStop();
            }
          }}
        >
          {streaming ? (
            <>
              <Stop />
              <span className="w-2" />
              Complete Recording
            </>
          ) : (
            <>
              <Record />
              <span className="w-2" />
              Start Recording
            </>
          )}
        </button>
        <div className="flex-grow" />
      </div>
    </div>
  );
}
