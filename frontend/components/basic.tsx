import classNames from "classnames";
import {
  ComponentPropsWithoutRef,
  Dispatch,
  ReactNode,
  SetStateAction,
} from "react";

interface ModalProps {
  isOpen: boolean;
  setIsOpen: Dispatch<SetStateAction<boolean>>;
  children?: ReactNode;
}

export function Modal<Props>({
  isOpen,
  setIsOpen,
  children,
  ...props
}: ModalProps & ComponentPropsWithoutRef<"div">) {
  return (
    <div
      className={classNames({
        modal: true,
        "is-active": isOpen,
      })}
    >
      <div className="modal-background" onClick={() => setIsOpen(false)}></div>
      <div className="modal-content" {...props}>
        {children}
      </div>
    </div>
  );
}
