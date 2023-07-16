import React from "react";
import { render, act, fireEvent, within } from "@testing-library/react";

import Toaster, { ToasterInterface } from "./Toaster";

describe("Toaster", () => {
  it("renders error and info toasts, which can be dismissed", () => {
    const toasterRef = { current: null as ToasterInterface | null };
    const toaster = render(<Toaster ref={toasterRef} />);
    expect(toaster.asFragment()).toMatchInlineSnapshot(`
      <DocumentFragment>
        <div
          class="fixed left-1/2 bottom-0 flex flex-col-reverse"
        />
      </DocumentFragment>
    `);

    act(() => {
      toasterRef.current.info("This is an info toast");
    });

    expect(toaster.asFragment()).toMatchInlineSnapshot(`
      <DocumentFragment>
        <div
          class="fixed left-1/2 bottom-0 flex flex-col-reverse"
        >
          <div
            class="left-1/2 -translate-x-1/2 p-2 first-of-type:border-b-0 border-2 last-of-type:rounded-t-md flex text-blue-900 bg-blue-100 border-blue-300"
            data-testid="toast-0"
          >
            <div
              class="flex-grow"
              data-testid="toast-label-0"
            >
              This is an info toast
            </div>
            <button
              class="cursor-pointer ml-4 font-bold inline opacity-50 hover:opacity-100"
              data-testid="dismiss-toast-0"
              title="Dismiss"
            >
              <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M6 18L18 6M6 6l12 12"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </button>
          </div>
        </div>
      </DocumentFragment>
    `);

    expect(
      toaster
        .getAllByTestId(/^toast-\d+$/)
        .map((toast) => `${toast.className}: ${toast.textContent}`),
    ).toEqual([
      "left-1/2 -translate-x-1/2 p-2 first-of-type:border-b-0 border-2 last-of-type:rounded-t-md flex text-blue-900 bg-blue-100 border-blue-300: This is an info toast",
    ]);

    act(() => {
      toasterRef.current.error("This is an error toast");
    });

    expect(
      toaster
        .getAllByTestId(/^toast-\d+$/)
        .map((toast) => `${toast.className}: ${toast.textContent}`),
    ).toEqual([
      "left-1/2 -translate-x-1/2 p-2 first-of-type:border-b-0 border-2 last-of-type:rounded-t-md flex text-blue-900 bg-blue-100 border-blue-300: This is an info toast",
      "left-1/2 -translate-x-1/2 p-2 first-of-type:border-b-0 border-2 last-of-type:rounded-t-md flex text-red-900 bg-red-100 border-red-300: This is an error toast",
    ]);

    const dismissInfo = within(
      toaster
        .getAllByTestId(/^toast-label-\d+$/)
        .find((t) => t.textContent === "This is an info toast").parentElement,
    ).getByRole("button", { name: "Dismiss" });

    fireEvent.click(dismissInfo);

    expect(
      toaster
        .getAllByTestId(/^toast-\d+$/)
        .map((toast) => `${toast.className}: ${toast.textContent}`),
    ).toEqual([
      "left-1/2 -translate-x-1/2 p-2 first-of-type:border-b-0 border-2 last-of-type:rounded-t-md flex text-red-900 bg-red-100 border-red-300: This is an error toast",
    ]);

    const dismissError = within(
      toaster
        .getAllByTestId(/^toast-label-\d+$/)
        .find((t) => t.textContent === "This is an error toast").parentElement,
    ).getByRole("button", { name: "Dismiss" });

    fireEvent.click(dismissError);

    expect(
      toaster
        .queryAllByTestId(/^toast-\d+$/)
        .map((toast) => `${toast.className}: ${toast.textContent}`),
    ).toEqual([]);
  });

  it("dismisses toasts with the same uniqueKey", () => {
    const toasterRef = { current: null as ToasterInterface | null };
    const toaster = render(<Toaster ref={toasterRef} />);

    act(() => {
      toasterRef.current.info("info toast 1", null, "key1");
      toasterRef.current.error("error toast 2", null, "key2");
      toasterRef.current.info("info toast 3", null, null);
    });

    expect(
      toaster.getAllByTestId(/^toast-\d+$/).map((toast) => toast.textContent),
    ).toEqual(["info toast 1", "error toast 2", "info toast 3"]);

    act(() => {
      toasterRef.current.error("replacement error toast 1", null, "key1");
      toasterRef.current.info("replacement info toast 2", null, "key2");
      toasterRef.current.info("info toast 4", null, null);
    });

    expect(
      toaster.getAllByTestId(/^toast-\d+$/).map((toast) => toast.textContent),
    ).toEqual([
      "info toast 3",
      "replacement error toast 1",
      "replacement info toast 2",
      "info toast 4",
    ]);
  });

  it("renders and responds to toast actions", () => {
    const toasterRef = { current: null as ToasterInterface | null };
    const toaster = render(<Toaster ref={toasterRef} />);
    const handleCb = jest.fn();

    act(() => {
      toasterRef.current.info("This is an info toast", {
        text: "Foobarify",
        cb: handleCb,
      });
    });

    expect(toaster.asFragment()).toMatchInlineSnapshot(`
      <DocumentFragment>
        <div
          class="fixed left-1/2 bottom-0 flex flex-col-reverse"
        >
          <div
            class="left-1/2 -translate-x-1/2 p-2 first-of-type:border-b-0 border-2 last-of-type:rounded-t-md flex text-blue-900 bg-blue-100 border-blue-300"
            data-testid="toast-0"
          >
            <div
              class="flex-grow"
              data-testid="toast-label-0"
            >
              This is an info toast
            </div>
            <button
              class="cursor-pointer ml-4 font-bold inline opacity-50 hover:opacity-100"
              data-testid="action-toast-0"
            >
              Foobarify
            </button>
            <button
              class="cursor-pointer ml-4 font-bold inline opacity-50 hover:opacity-100"
              data-testid="dismiss-toast-0"
              title="Dismiss"
            >
              <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                viewBox="0 0 24 24"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M6 18L18 6M6 6l12 12"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            </button>
          </div>
        </div>
      </DocumentFragment>
    `);

    const action = within(
      toaster
        .getAllByTestId(/^toast-label-\d+$/)
        .find((t) => t.textContent === "This is an info toast").parentElement,
    ).getByRole("button", { name: "Foobarify" });

    fireEvent.click(action);

    expect(handleCb).toHaveBeenCalledTimes(1);

    expect(
      toaster
        .queryAllByTestId(/^toast-\d+$/)
        .map((toast) => `${toast.className}: ${toast.textContent}`),
    ).toEqual([]);
  });
});
