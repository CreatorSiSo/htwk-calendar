import type { FunctionComponent, JSX } from "preact";

export const IconButton: FunctionComponent<
  JSX.HTMLAttributes<HTMLButtonElement>
> = ({ children, class: classes, ...props }) => {
  return (
    <button
      class={
        "h-11 w-9 flex items-center justify-center text-neutral-800 " + classes
      }
      {...props}
    >
      {children}
    </button>
  );
};
