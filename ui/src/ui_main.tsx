import { UiState } from "oxygen-core";
import React, { useState, useCallback, useRef, useReducer } from "react";

import Toaster, { ToasterInterface } from "./toaster";
import ClipList from "./clip_list";
import RecordTab from "./record_tab";
import CurrentClip from "./current_clip";

export default function Main({ inMemory }: { inMemory: boolean }) {
  // Hack to force a re-render when the state changes.
  const [, forceUpdate] = useReducer(() => ({}), []);
  const [uiState] = useState(() => new UiState(forceUpdate, inMemory));
  const toaster = useRef<ToasterInterface>(null);

  const drawCurrentClipWaveform = useCallback(
    (width: number, height: number) =>
      uiState.drawCurrentClipWaveform(width, height),
    [uiState]
  );

  const handlePlay = useCallback(
    (cb: () => void) => {
      uiState.play(cb);
    },
    [uiState]
  );

  const handleStop = useCallback(() => uiState.stop(), [uiState]);

  const handleRename = useCallback(
    (name: string) => {
      try {
        const prevName = uiState.currentClip.name;
        if (name !== prevName) {
          uiState.renameCurrentClip(name);
          toaster.current.info(`Renamed "${prevName}" to "${name}"`);
        }
      } catch (err) {
        if (err instanceof Error) {
          // TODO: stable interface for error messages and/or tests
          if (err.message == "UNIQUE constraint failed: clips.name") {
            toaster.current.error("This name is taken by another clip.");
          } else {
            toaster.current.error(
              "Something went wrong when renaming this clip."
            );
          }
        }
      }
    },
    [uiState]
  );

  const handleDelete = useCallback(() => {
    const prevName = uiState.currentClip.name;
    uiState.deleteCurrentClip();
    toaster.current.info(
      `Deleted "${prevName}"`,
      {
        text: "Undo",
        cb: () => {
          uiState.undeleteCurrentClip();
        },
      },
      "undoDeleteCurrentClip"
    );
  }, [uiState]);

  const handleRecord = useCallback(() => {
    uiState.record();
  }, [uiState]);

  const handleSetTabRecord = useCallback(() => {
    uiState.setCurrentTabRecord();
  }, [uiState]);

  const handleSetCurrentClipId = useCallback(
    (clipId: number) => {
      uiState.setCurrentClipId(clipId);
    },
    [uiState]
  );

  return (
    <div className="w-screen h-screen flex flex-row">
      <Toaster ref={toaster} />
      <ClipList
        clips={uiState.getClips().reverse()}
        recordTabSelected={uiState.recordTabSelected}
        currentClipId={uiState.currentClipId}
        onSetCurrentTabRecord={handleSetTabRecord}
        onSetCurrentClipId={handleSetCurrentClipId}
      />
      {uiState.currentClipId != null && (
        <CurrentClip
          clip={uiState.currentClip}
          drawCurrentClipWaveform={drawCurrentClipWaveform}
          time={uiState.time}
          timePercent={uiState.timePercent}
          streaming={uiState.streaming}
          onPlay={handlePlay}
          onStop={handleStop}
          onRename={handleRename}
          onDelete={handleDelete}
        />
      )}
      {uiState.recordTabSelected && (
        <RecordTab
          streaming={uiState.streaming}
          onRecord={handleRecord}
          onStop={handleStop}
        />
      )}
    </div>
  );
}