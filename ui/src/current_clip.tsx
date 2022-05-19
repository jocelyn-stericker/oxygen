import { UiState } from "oxygen-core";
import cx from "classnames";
import React, { useState, useEffect, useRef, useCallback } from "react";
import { Pause, Play, Delete } from "./icons";
import { ToasterInterface } from "./toaster";

export default function CurrentClip({
  uiState,
  toaster,
}: {
  uiState: UiState;
  toaster: React.MutableRefObject<ToasterInterface>;
}) {
  const clip = uiState.currentClip;
  const [temporaryName, setTemporaryName] = useState(clip.name);
  useEffect(() => {
    setTemporaryName(clip.name);
  }, [clip.name]);
  const canvas = useRef<HTMLCanvasElement>(null);
  const canvasContainer = useRef<HTMLDivElement>(null);

  const redraw = useCallback(() => {
    console.time("draw");
    const rect = canvas.current.parentElement.getBoundingClientRect();
    canvas.current.width = rect.width * devicePixelRatio;
    canvas.current.height = rect.height * devicePixelRatio;
    canvas.current.style.width = `${rect.width * devicePixelRatio}px`;
    canvas.current.style.height = `${rect.height * devicePixelRatio}px`;
    canvas.current.style.transform = `scale(${1 / devicePixelRatio})`;
    canvas.current.style.transformOrigin = "top left";

    const buffer = uiState.drawCurrentClipWaveform(
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
    console.timeEnd("draw");
  }, [uiState]);

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

  useEffect(redraw, [redraw, uiState.currentClipId]);

  return (
    <div className="flex flex-col flex-grow">
      <div className="flex flex-row">
        <input
          className="self-center p-2 m-2 text-m font-bold overflow-ellipses overflow-hidden border-2 border-purple-200 rounded-md focus:border-purple-900 outline-purple-900 text-purple-900 flex-grow transition-all"
          value={temporaryName}
          onChange={(ev) => {
            setTemporaryName(ev.currentTarget.value);
          }}
          onBlur={() => {
            const name = temporaryName.trim();
            if (name != "") {
              try {
                if (name !== clip.name) {
                  uiState.renameCurrentClip(name);
                  toaster.current.info(`Renamed "${clip.name}" to "${name}"`);
                }
              } catch (err) {
                if (err instanceof Error) {
                  // TODO: stable interface for error messages and/or tests
                  if (err.message == "UNIQUE constraint failed: clips.name") {
                    toaster.current.error(
                      "This name is taken by another clip."
                    );
                  } else {
                    toaster.current.error(
                      "Something went wrong when renaming this clip."
                    );
                  }
                }
              }
            }
            setTemporaryName(uiState.currentClip.name);
          }}
        />
        <button
          className="p-2 m-2 ml-0 text-purple-900 cursor-pointer border-2 border-transparent hover:border-red-900 rounded-full hover:bg-red-100 hover:text-red-900"
          title="Delete this clip"
          onClick={(ev) => {
            uiState.deleteCurrentClip();
            toaster.current.info(
              `Deleted "${clip.name}"`,
              {
                text: "Undo",
                cb: () => {
                  uiState.undeleteCurrentClip();
                },
              },
              "undoDeleteCurrentClip"
            );
            ev.preventDefault();
          }}
        >
          <Delete />
        </button>
      </div>
      <div className="flex-grow relative overflow-hidden" ref={canvasContainer}>
        <canvas className="absolute w-full h-full" ref={canvas} />
      </div>
      <div className="flex flex-row mb-4">
        <div className="flex-grow" />
        <button
          className={cx(
            "p-4 rounded-md m-auto text-lg flex border-2",
            uiState.streaming
              ? "bg-white border-purple-900 text-purple-900 hover:bg-purple-100"
              : "bg-purple-900 text-white hover:bg-purple-800"
          )}
          onClick={(ev) => {
            ev.preventDefault();
            if (!uiState.streaming) {
              uiState.play(() => {
                uiState.stop();
              });
            } else {
              uiState.stop();
            }
          }}
        >
          {uiState.streaming ? (
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
      </div>
    </div>
  );
}
