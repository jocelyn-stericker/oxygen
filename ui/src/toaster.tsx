import React, {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import cx from "classnames";
import { Dismiss } from "./icons";

type ToastType = "error" | "info";
interface Toast {
  toastType: ToastType;
  id: number;
  text: string;
}

export interface ToasterInterface {
  error: (msg: string) => void;
  info: (msg: string) => void;
}

const Toaster = forwardRef<ToasterInterface>(function Toaster(_props, ref) {
  const [toasts, setToasts] = useState<Array<Toast>>([]);
  const nextId = useRef(0);

  const mounted = useRef(true);
  useEffect(() => {
    return () => {
      mounted.current = false;
    };
  }, []);

  const addToast = useCallback((text: string, toastType: ToastType) => {
    const id = nextId.current;
    setToasts((toasts) => [...toasts, { text, toastType, id }]);
    nextId.current += 1;

    setTimeout(() => {
      setToasts((toasts) => toasts.filter((toast) => toast.id !== id));
    }, 4000);
  }, []);

  useImperativeHandle(
    ref,
    () => ({
      error: (msg: string) => {
        addToast(msg, "error");
      },
      info: (msg: string) => {
        addToast(msg, "info");
      },
    }),
    [addToast]
  );

  return (
    <div className="fixed left-1/2">
      {toasts.map((toast) => (
        <div
          className={cx(
            "left-1/2 -translate-x-1/2 p-2 first-of-type:border-t-0 border-2 last-of-type:rounded-b-md flex",
            toast.toastType === "error" &&
              "text-red-900 bg-red-100 border-red-300",
            toast.toastType === "info" &&
              "text-blue-900 bg-blue-100 border-blue-300"
          )}
          key={toast.id}
        >
          {toast.text}
          <button
            className="cursor-pointer ml-4 font-bold inline opacity-50"
            title="Dismiss"
            onClick={(ev) => {
              ev.preventDefault();
              const id = toast.id;
              setToasts((toasts) => toasts.filter((toast) => toast.id !== id));
            }}
          >
            <Dismiss />
          </button>
        </div>
      ))}
    </div>
  );
});

export default Toaster;
