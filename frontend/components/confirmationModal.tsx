import classNames from "classnames";
import { Dispatch, ReactNode, SetStateAction, useState } from "react";
import { Modal } from "./basic";

export interface ConfirmationModalProps<V, E> {
  isOpen: boolean;
  setIsOpen: Dispatch<SetStateAction<boolean>>;
  children?: ReactNode;
  actionName: string;
  onAccept: () => Promise<V>;
  onSuccess?: (response: V) => void;
  onError?: (error: E) => void;
}

export function ConfirmationModal<V, E>({
  isOpen,
  setIsOpen,
  children,
  actionName,
  onAccept,
  onSuccess,
  onError,
}: ConfirmationModalProps<V, E>) {
  const [isLoading, setIsLoading] = useState(false);

  return (
    <Modal isOpen={isOpen} setIsOpen={setIsOpen} style={{ width: "22rem" }}>
      <div className="box">
        <p className="block">{children}</p>
        <div className="field is-grouped is-grouped-right mb-0">
          <div className="control">
            <button className="button" onClick={() => setIsOpen(false)}>
              Cancel
            </button>
          </div>
          <div className="control">
            <button
              className={classNames({
                button: true,
                "is-danger": true,
                "is-loading": isLoading,
              })}
              onClick={() => {
                setIsLoading(true);
                onAccept()
                  .then((response) => {
                    setIsOpen(false);
                    setIsLoading(false);
                    if (onSuccess) onSuccess(response);
                  })
                  .catch((error) => {
                    if (onError) onError(error as E);
                  });
              }}
            >
              {actionName}
            </button>
          </div>
        </div>
      </div>
    </Modal>
  );
}
