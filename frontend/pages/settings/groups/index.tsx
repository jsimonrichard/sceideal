import SettingsLayout, {
  AdminSettingPage,
  GeneralSettingPage,
} from "@/components/settings_layout";
import { faAdd, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { yupResolver } from "@hookform/resolvers/yup";
import { Dispatch, ReactElement, SetStateAction, useState } from "react";
import * as yup from "yup";
import { Controller, useForm } from "react-hook-form";
import { AsyncStatus, useAsync } from "@/components/hooks";
import axios, { isAxiosError } from "axios";
import classNames from "classnames";
import { CreateGroup, Group } from "@/shared-types";
import { useRouter } from "next/router";

export default function GroupsSettings() {
  const router = useRouter();

  const [isCreateGroupModalOpen, setIsCreateGroupModalOpen] = useState(false);

  const {
    status,
    value: groups,
    error,
    execute,
  } = useAsync<void, Group[]>(
    () => axios.get("/api/group/a"),
    () => {},
    () => {},
    true
  );

  let rows;
  if (status == AsyncStatus.Idle || status == AsyncStatus.Pending) {
    rows = (
      <tr>
        <td colSpan={6} className="loader-row"></td>
      </tr>
    );
  } else if (status == AsyncStatus.Error || !groups) {
    rows = (
      <tr>
        <td colSpan={6} className="has-text-danger">
          {error?.message}
        </td>
      </tr>
    );
  } else {
    if (groups.length == 0) {
      rows = (
        <tr>
          <td colSpan={6} style={{ textAlign: "center" }}>
            No users have been created
          </td>
        </tr>
      );
    } else {
      rows = groups.map((group) => (
        <tr
          key={group.id}
          onClick={() => {
            router.push(`/settings/groups/${group.id}`);
          }}
          style={{ cursor: "pointer" }}
        >
          <td>{group.id}</td>
          <td>{group.name}</td>
          <td>{group.description}</td>
          <td>{group.public ? "Public" : "Private"}</td>
        </tr>
      ));
    }
  }

  return (
    <div>
      <table className="table is-fullwidth is-striped is-hoverable">
        <thead>
          <tr>
            <th>ID</th>
            <th>Name</th>
            <th>Description</th>
            <th>Visibility</th>
          </tr>
        </thead>

        <tbody>{rows}</tbody>
      </table>

      <button
        className="button is-medium is-link is-inverted"
        onClick={() => setIsCreateGroupModalOpen(true)}
      >
        <FontAwesomeIcon
          icon={faAdd}
          className="mr-3"
          style={{
            height: "1em",
          }}
        />
        <span>Create Group</span>
      </button>

      <CreateGroupModal
        isOpen={isCreateGroupModalOpen}
        setIsOpen={setIsCreateGroupModalOpen}
        onSuccess={execute}
      />
    </div>
  );
}

GroupsSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={AdminSettingPage.Groups}>{page}</SettingsLayout>
  );
};

interface CreateGroupModalProps {
  isOpen: boolean;
  setIsOpen: Dispatch<SetStateAction<boolean>>;
  onSuccess?: () => void;
}

function CreateGroupModal({
  isOpen,
  setIsOpen,
  onSuccess,
}: CreateGroupModalProps) {
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
    defaultValues: { public: false },
    mode: "onTouched",
    resolver: yupResolver(formSchema),
  });

  const { execute, status, error } = useAsync<
    yup.InferType<typeof formSchema>,
    CreateGroup
  >(
    (data) => axios.post("/api/group/a", data),
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
      <div className="modal-content">
        <div className="box">
          <h2 className="title">Create a Group</h2>
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
                  <input
                    type="checkbox"
                    className="switch"
                    checked={value}
                    onChange={onChange}
                  />
                  <label className="label">
                    Public
                    <span
                      className="icon has-tooltip-right has-tooltip-multiline ml-1"
                      data-tooltip="If public, this group can be seen by anyone.
                                    Otherwise, students must be added via a group
                                    code / join link or be added manually."
                    >
                      <FontAwesomeIcon icon={faInfoCircle} width="0.8rem" />
                    </span>
                  </label>
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
                  Create
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
      </div>
      <button
        className="modal-close is-large"
        aria-label="close"
        onClick={() => setIsOpen(false)}
      ></button>
    </div>
  );
}
