import React from "react";
import cx from "classnames";

import { Record, Stop } from "./icons";
import AudioView from "./audio_view";

export default function RecordTab({
  drawCurrentClipWaveform,
  streaming,
  onRecord,
  onStop,
}: {
  drawCurrentClipWaveform: (width: number, height: number) => Buffer | null;
  streaming: boolean;
  onRecord: () => void;
  onStop: () => void;
}) {
  return (
    <div className="flex flex-col flex-grow">
      <AudioView
        drawCurrentClipWaveform={drawCurrentClipWaveform}
        timePercent={1}
        streaming={streaming}
      />
      <div className="flex flex-row mb-4">
        <div className="flex-grow" />
        <button
          data-testid="toggle-record"
          className={cx(
            "p-4 rounded-md m-auto text-lg flex border-2",
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
