import React, { useCallback, useEffect, useRef, useState } from "react";
import cx from "classnames";
import { RenderMode, JsSegment } from "oxygen-core";
import { Spectrogram } from "./icons";

export default function AudioView({
  drawCurrentClip,
  streaming,
  time,
  clipId,
  transcribe,
  onSeek,
  onSetRenderMode,
  renderMode,
  timeStart,
  timeEnd,
}: {
  drawCurrentClip: (width: number, height: number) => Buffer | null;
  streaming?: boolean;
  time: number;
  clipId?: bigint | number;
  transcribe?: () => Promise<JsSegment[]>;
  onSeek: (time: number) => void;
  onSetRenderMode: (renderMode: RenderMode) => void;
  renderMode: RenderMode;
  timeStart: number;
  timeEnd: number;
}) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const canvasContainer = useRef<HTMLDivElement>(null);
  const duration = timeEnd - timeStart;

  const redraw = useCallback(() => {
    const parent = canvas.current?.parentElement;
    if (!parent) {
      // called one last time on dismount, before the observer disconnects.
      return;
    }

    const rect = parent.getBoundingClientRect();
    canvas.current.width = rect.width * devicePixelRatio;
    canvas.current.height = rect.height * devicePixelRatio;
    canvas.current.style.width = `${rect.width * devicePixelRatio}px`;
    canvas.current.style.height = `${rect.height * devicePixelRatio}px`;
    canvas.current.style.transform = `scale(${1 / devicePixelRatio})`;
    canvas.current.style.transformOrigin = "top left";

    const buffer = drawCurrentClip(canvas.current.width, canvas.current.height);

    const array = new Uint8ClampedArray(buffer);
    if (array.length > 0) {
      const image = new ImageData(
        array,
        canvas.current.width,
        canvas.current.height
      );
      const context = canvas.current.getContext("2d");
      context.putImageData(image, 0, 0);
    }
  }, [drawCurrentClip]);

  const [transcription, setTranscription] = useState<
    Array<{
      t0: number;
      t1: number;
      segment: string;
    }>
  >(null);

  useEffect(() => {
    let expired = false;
    setTranscription([]);

    (async () => {
      if (transcribe) {
        const transcription = await transcribe();
        if (!expired) {
          setTranscription(transcription);
        }
      }
    })();

    return () => {
      expired = true;
    };
  }, [clipId, transcribe]);

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

  useEffect(() => {
    if (streaming) {
      const interval = setInterval(() => {
        redraw();
      }, 100);
      return () => {
        clearInterval(interval);
      };
    }
  }, [redraw, streaming]);

  useEffect(redraw, [redraw, clipId, renderMode]);

  return (
    <>
      <div className="flex-grow relative overflow-hidden" ref={canvasContainer}>
        <canvas
          data-testid="current-clip-view"
          className="absolute w-full h-full"
          ref={canvas}
          onClick={(ev) => {
            const rect = ev.currentTarget.getBoundingClientRect();
            onSeek(
              ((ev.clientX - rect.left) / rect.width) * (timeEnd - timeStart)
            );
          }}
        />
        <div
          data-testid="current-clip-cursor"
          className="absolute w-[1px] bg-blue-400 h-full"
          style={{
            left: `${((time - timeStart) / (timeEnd - timeStart)) * 100}%`,
          }}
        />
        <input
          type="checkbox"
          data-testid="current-clip-spectrogram"
          title="Toggle spectrogram"
          className="invisible"
          checked={renderMode === RenderMode.Spectrogram}
          onChange={(ev) => {
            ev.preventDefault();
            if (ev.target.checked) {
              onSetRenderMode(RenderMode.Spectrogram);
            } else {
              onSetRenderMode(RenderMode.Waveform);
            }
          }}
          id="toggle-spectrogram"
        ></input>
        <label
          htmlFor="toggle-spectrogram"
          className={cx(
            "absolute right-0 bottom-0 p-2 m-2 ml-0 text-purple-900 cursor-pointer border-2 hover:border-purple-900 rounded-full hover:bg-purple-100 hover:text-purple-900",
            renderMode === RenderMode.Spectrogram
              ? "border-purple-900"
              : "border-transparent"
          )}
        >
          <Spectrogram />
        </label>
      </div>
      <div className="m-2 w-full h-10 relative">
        {transcription?.map((segment, i) => (
          <svg
            key={i}
            width={`${
              ((Math.min(duration, segment.t1 - timeStart) -
                (segment.t0 - timeStart)) /
                duration) *
              100
            }%`}
            height={30}
            style={{
              position: "absolute",
              left: `${((segment.t0 - timeStart) / duration) * 100}%`,
            }}
          >
            <g>
              <text
                y="50%"
                x="0"
                textLength="100%"
                fontSize={10}
                lengthAdjust="spacingAndGlyphs"
              >
                {segment.segment}
              </text>
            </g>
          </svg>
        ))}
      </div>
    </>
  );
}
