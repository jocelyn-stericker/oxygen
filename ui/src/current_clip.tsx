import { JsClipMeta, Segment } from "oxygen-core";
import cx from "classnames";
import React, { useState, useEffect } from "react";
import { Pause, Play, Delete } from "./icons";
import AudioView from "./audio_view";

export default function CurrentClip({
  clip,
  drawCurrentClipWaveform,
  transcribe,
  time,
  timePercent,
  duration,
  streaming,
  onPlay,
  onStop,
  onSeek,
  onRename,
  onDelete,
}: {
  clip: JsClipMeta;
  drawCurrentClipWaveform: (width: number, height: number) => Buffer | null;
  transcribe: () => Segment[];
  time: number;
  timePercent: number;
  duration: number;
  streaming: boolean;
  onPlay: (cb: () => void) => void;
  onStop: () => void;
  onSeek: (timePercent: number) => void;
  onRename: (name: string) => void;
  onDelete: () => void;
}) {
  const [temporaryName, setTemporaryName] = useState(clip.name);
  useEffect(() => {
    setTemporaryName(clip.name);
  }, [clip.name]);

  return (
    <div className="flex flex-col flex-grow overflow-hidden">
      <div className="flex flex-row">
        <input
          data-testid="current-clip-name"
          className="self-center p-2 m-2 text-m font-bold overflow-ellipses overflow-hidden border-2 border-purple-200 rounded-md focus:border-purple-900 outline-purple-900 text-purple-900 flex-grow transition-all"
          value={temporaryName}
          onChange={(ev) => {
            setTemporaryName(ev.currentTarget.value);
          }}
          onBlur={() => {
            const name = temporaryName.trim();
            if (name !== "") {
              onRename(name);
            }
            setTemporaryName(clip.name);
          }}
        />
        <button
          data-testid="current-clip-delete"
          className="p-2 m-2 ml-0 text-purple-900 cursor-pointer border-2 border-transparent hover:border-red-900 rounded-full hover:bg-red-100 hover:text-red-900"
          title="Delete this clip"
          onClick={(ev) => {
            onDelete();
            ev.preventDefault();
          }}
        >
          <Delete />
        </button>
      </div>
      <AudioView
        drawCurrentClipWaveform={drawCurrentClipWaveform}
        timePercent={timePercent}
        duration={duration}
        clipId={clip.id}
        transcribe={transcribe}
        onSeek={onSeek}
      />
      <div className="flex flex-row mb-4">
        <div
          className="flex self-center font-mono text-purple-900 mx-2 w-20"
          data-testid="current-clip-time"
        >
          {time.toFixed(2).padStart(6, "0")}
        </div>
        <div className="flex-grow" />
        <button
          data-testid="current-clip-toggle-playback"
          className={cx(
            "px-4 rounded-md m-auto text-lg flex border-2 h-16 items-center",
            streaming
              ? "bg-white border-purple-900 text-purple-900 hover:bg-purple-100"
              : "bg-purple-900 text-white hover:bg-purple-800"
          )}
          onClick={(ev) => {
            ev.preventDefault();
            if (!streaming) {
              onPlay(() => {
                onStop();
              });
            } else {
              onStop();
            }
          }}
        >
          {streaming ? (
            <>
              <Pause />
              <span className="w-2" />
              Pause
            </>
          ) : (
            <>
              <Play />
              <span className="w-2" />
              Play
            </>
          )}
        </button>
        <div className="flex-grow" />
        <div className="w-20 mx-2" />
      </div>
    </div>
  );
}
