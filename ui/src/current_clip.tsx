import { JsClipMeta } from "oxygen-core";
import cx from "classnames";
import React, { useState, useEffect, useRef, useCallback } from "react";
import { Pause, Play, Delete } from "./icons";

export default function CurrentClip({
  clip,
  drawCurrentClipWaveform,
  time,
  timePercent,
  streaming,
  onPlay,
  onStop,
  onRename,
  onDelete,
}: {
  clip: JsClipMeta;
  drawCurrentClipWaveform: (width: number, height: number) => Buffer | null;
  time: number;
  timePercent: number;
  streaming: boolean;
  onPlay: (cb: () => void) => void;
  onStop: () => void;
  onRename: (name: string) => void;
  onDelete: () => void;
}) {
  const [temporaryName, setTemporaryName] = useState(clip.name);
  useEffect(() => {
    setTemporaryName(clip.name);
  }, [clip.name]);
  const canvas = useRef<HTMLCanvasElement>(null);
  const canvasContainer = useRef<HTMLDivElement>(null);

  const redraw = useCallback(() => {
    const rect = canvas.current.parentElement.getBoundingClientRect();
    canvas.current.width = rect.width * devicePixelRatio;
    canvas.current.height = rect.height * devicePixelRatio;
    canvas.current.style.width = `${rect.width * devicePixelRatio}px`;
    canvas.current.style.height = `${rect.height * devicePixelRatio}px`;
    canvas.current.style.transform = `scale(${1 / devicePixelRatio})`;
    canvas.current.style.transformOrigin = "top left";

    const buffer = drawCurrentClipWaveform(
      canvas.current.width,
      canvas.current.height
    );

    const array = new Uint8ClampedArray(buffer);
    const image = new ImageData(
      array,
      canvas.current.width,
      canvas.current.height
    );
    const context = canvas.current.getContext("2d");
    context.putImageData(image, 0, 0);
  }, [drawCurrentClipWaveform]);

  useEffect(() => {
    // ResizeObserver calls immediately on observe, so we need to work around that.
    const state = { didInit: false };
    const observer = new ResizeObserver(() => {
      if (state.didInit) {
        redraw();
      } else {
        state.didInit = true;
      }
    });
    observer.observe(canvasContainer.current);
    return () => {
      observer.disconnect();
    };
  }, [redraw]);

  useEffect(redraw, [redraw, clip.id]);

  return (
    <div className="flex flex-col flex-grow">
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
      <div className="flex-grow relative overflow-hidden" ref={canvasContainer}>
        <canvas
          data-testid="current-clip-view"
          className="absolute w-full h-full"
          ref={canvas}
        />
        <div
          data-testid="current-clip-cursor"
          className="absolute w-[1px] bg-blue-400 h-full"
          style={{ left: `${timePercent * 100}%` }}
        />
      </div>
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
            "p-4 rounded-md m-auto text-lg flex border-2",
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
