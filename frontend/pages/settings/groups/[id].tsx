import { AsyncStatus, useAsync } from "@/components/hooks";
import { Group, UpdateGroup } from "@/shared-types";
import { yupResolver } from "@hookform/resolvers/yup";
import axios, { isAxiosError } from "axios";
import classNames from "classnames";
import { useRouter } from "next/router";
import {
  Dispatch,
  ReactElement,
  SetStateAction,
  useEffect,
  useState,
} from "react";
import { Controller, useForm } from "react-hook-form";
import * as yup from "yup";
import GroupsSettings from ".";
import GoBackLayout from "@/components/go_back_layout";

export default function GroupSettings() {
  const router = useRouter();
  const { id } = router.query;

  // Get group data
  const {
    status,
    value: group,
    error,
    execute,
  } = useAsync(() => axios.get(`/api/group/a/${id}`));

  useEffect(() => {
    if (id !== undefined) {
      execute();
    }
  }, [id]);

  if (
    id === undefined ||
    Array.isArray(id) ||
    id.length === 0 ||
    isNaN(id as unknown as number)
  ) {
    return <p className="has-text-danger">Invalid user ID</p>;
  }

  // Main content
  return (
    <>
      <div className="columns is-variable is-1-mobile is-3-desktop is-5-widescreen mt-5">
        <div className="column">
          {(() => {
            // Handle async errors
            if (status === AsyncStatus.Idle || status === AsyncStatus.Pending) {
              return <div className="page-loader"></div>;
            } else if (status === AsyncStatus.Error || !group) {
              return <p className="has-text-danger">{error?.message}</p>;
            } else {
              return <EditGroupForm group={group} />;
            }
          })()}
        </div>
        <div className="column" style={{ borderLeft: "solid #00000033 1px" }}>
          <EditGroupUsers groupId={id} />
        </div>
      </div>
    </>
  );
}

GroupSettings.getLayout = (page: ReactElement) => (
  <GoBackLayout backTo="/settings/groups">{page}</GoBackLayout>
);

interface EditGroupFormProps {
  group: Group;
}

function EditGroupForm({ group }: EditGroupFormProps) {
  const router = useRouter();

  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);

  const formSchema = yup.object().shape({
    name: yup.string().required("Name is required"),
    description: yup.string().nullable(),
    public: yup.boolean().required("Visibility is required"),
  });

  const {
    register,
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<yup.InferType<typeof formSchema>>({
    values: group,
    mode: "onTouched",
    resolver: yupResolver(formSchema),
  });

  const { execute, status, error } = useAsync<
    yup.InferType<typeof formSchema>,
    UpdateGroup
  >((data) => axios.put(`/api/group/a/${group.id}`, data));

  return (
    <>
      <div className="">
        <form onSubmit={handleSubmit(execute)}>
          <div className="field">
            <label className="label">Name*</label>
            <div className="control">
              <input
                type="text"
                {...register("name")}
                className={classNames({
                  input: true,
                  "is-danger": errors.name,
                })}
              />
            </div>
            {errors.name && (
              <p className="help is-danger">{errors.name.message}</p>
            )}
          </div>

          <div className="field">
            <label className="label">Description</label>
            <div className="control">
              <textarea
                rows={6}
                {...register("description")}
                className={classNames({
                  textarea: true,
                  "is-danger": errors.description,
                })}
              ></textarea>
            </div>
            {errors.description && (
              <p className="help is-danger">{errors.description.message}</p>
            )}
          </div>

          <Controller
            control={control}
            name="public"
            render={({ field: { onChange, value } }) => (
              <div className="field" onClick={() => onChange(!value)}>
                <input type="checkbox" className="switch" checked={value} />
                <label className="label">Public</label>
                {errors.public && (
                  <p className="help is-danger">{errors.public.message}</p>
                )}
              </div>
            )}
          />

          <div className="field is-grouped is-grouped-right">
            <div className="control">
              <button
                className={classNames({
                  button: true,
                  "is-primary": true,
                  "is-loading": status == AsyncStatus.Pending,
                })}
              >
                Update
              </button>
            </div>
            <div className="control">
              <button
                className="button is-danger"
                type="button"
                onClick={() => setIsDeleteModalOpen(true)}
              >
                Delete
              </button>
            </div>
          </div>
        </form>

        {error && (
          <p className="has-text-danger mt-3">
            {(error &&
              isAxiosError(error) &&
              typeof error.response?.data == "string" &&
              error.response.data) ||
              error.message}
          </p>
        )}
      </div>

      <DeleteGroupModal
        isOpen={isDeleteModalOpen}
        setIsOpen={setIsDeleteModalOpen}
        group={group}
        onSuccess={() => {
          router.push("/settings/groups");
        }}
      />
    </>
  );
}

interface DeleteGroupModalProps {
  isOpen: boolean;
  setIsOpen: Dispatch<SetStateAction<boolean>>;
  group: Group;
  onSuccess?: () => void;
}

function DeleteGroupModal({
  isOpen,
  setIsOpen,
  group,
  onSuccess,
}: DeleteGroupModalProps) {
  const { execute, status, error } = useAsync(
    () => axios.delete(`/api/group/a/${group.id}`),
    () => {
      setIsOpen(false);
      if (onSuccess) {
        onSuccess();
      }
    }
  );

  return (
    <div
      className={classNames({
        modal: true,
        "is-active": isOpen,
      })}
    >
      <div className="modal-background" onClick={() => setIsOpen(false)}></div>
      <div className="modal-content" style={{ width: "22em" }}>
        <div className="box">
          <p>
            Are you sure you want to delete <strong>{group.name}</strong>?
          </p>
          <div className="field is-grouped is-grouped-right mt-5">
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
                  "is-loading": status == AsyncStatus.Pending,
                })}
                onClick={execute}
              >
                Delete
              </button>
            </div>
          </div>
          {error && (
            <p className="has-text-danger mt-3">
              {(error &&
                isAxiosError(error) &&
                typeof error.response?.data == "string" &&
                error.response.data) ||
                error.message}
            </p>
          )}
        </div>
      </div>
      <button
        className="modal-close is-large"
        aria-label="close"
        onClick={() => setIsOpen(false)}
      ></button>
    </div>
  );
}

interface EditGroupUsersProps {
  groupId: string;
}

function EditGroupUsers({ groupId }: EditGroupUsersProps) {
  return (
    <div className="">
      <h2 className="title is-4">Users</h2>
    </div>
  );
}
