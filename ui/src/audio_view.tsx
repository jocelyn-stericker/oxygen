import React, { useCallback, useEffect, useRef } from "react";

export default function AudioView({
  drawCurrentClipWaveform,
  streaming,
  timePercent,
  clipId,
  onSeek,
}: {
  drawCurrentClipWaveform: (width: number, height: number) => Buffer | null;
  streaming?: boolean;
  timePercent: number;
  clipId?: bigint | number;
  onSeek: (timePercent: number) => void;
}) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const canvasContainer = useRef<HTMLDivElement>(null);

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

    const buffer = drawCurrentClipWaveform(
      canvas.current.width,
      canvas.current.height
    );

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

  useEffect(redraw, [redraw, clipId]);

  return (
    <div className="flex-grow relative overflow-hidden" ref={canvasContainer}>
      <canvas
        data-testid="current-clip-view"
        className="absolute w-full h-full"
        ref={canvas}
        onClick={(ev) => {
          const rect = ev.currentTarget.getBoundingClientRect();
          onSeek((ev.clientX - rect.left) / rect.width);
        }}
      />
      <div
        data-testid="current-clip-cursor"
        className="absolute w-[1px] bg-blue-400 h-full"
        style={{ left: `${timePercent * 100}%` }}
      />
    </div>
  );
}
