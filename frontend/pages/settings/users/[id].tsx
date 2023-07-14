import GoBackLayout from "@/components/goBackLayout";
import { useRouter } from "next/router";
import {
  Dispatch,
  ReactElement,
  SetStateAction,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { Controller, useForm } from "react-hook-form";
import * as yup from "yup";
import { yupResolver } from "@hookform/resolvers/yup";
import { AsyncStatus, useAsync } from "@/components/hooks";
import {
  AdminUpdateUser,
  CreateLocalUser,
  Group,
  MembershipData,
  PermissionLevel,
  PublicUserData,
  UpdateIsMemberOf,
  UserData,
} from "@/shared-types";
import axios, { isAxiosError } from "axios";
import _, { set } from "lodash";
import classNames from "classnames";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faEdit,
  faEnvelope,
  faPhone,
  faTrash,
} from "@fortawesome/free-solid-svg-icons";
import { useDebounce, useHoverDirty } from "react-use";
import { ConfirmationModal } from "@/components/confirmationModal";
import { Modal } from "@/components/basic";
import AsyncSearchSelect from "@/components/asyncSearchSelect";

export default function UserSettings() {
  const router = useRouter();
  const { id } = router.query;

  // Get user data
  const {
    status,
    value: user,
    error,
    execute,
  } = useAsync<void, UserData>(() => axios.get(`/api/user/a/${id}`));

  useEffect(() => {
    if (id !== undefined) {
      execute();
    }
  }, [id]);

  if (
    id !== undefined &&
    (Array.isArray(id) || id.length === 0 || isNaN(id as unknown as number))
  ) {
    return <p className="has-text-danger">Invalid user ID: {id}</p>;
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
            } else if (status === AsyncStatus.Error || !user) {
              return <p className="has-text-danger">{error?.message}</p>;
            } else {
              return <EditUserForm user={user} />;
            }
          })()}
        </div>
        <div className="column" style={{ borderLeft: "solid #00000033 1px" }}>
          {(() => {
            if (id === undefined) {
              return <div className="page-loader"></div>;
            } else {
              return <EditUserGroups userId={id} />;
            }
          })()}
        </div>
      </div>
    </>
  );
}

UserSettings.getLayout = (page: ReactElement) => {
  return <GoBackLayout backTo="/settings/users">{page}</GoBackLayout>;
};

interface EditUserProps {
  user: UserData;
}

function EditUserForm({ user }: EditUserProps) {
  const router = useRouter();

  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);

  const formSchema = yup.object().shape({
    fname: yup.string().required("First name is required"),
    lname: yup.string().required("Last name is required"),
    email_verified: yup.boolean().required("Missing Email Verification status"),
    phone_number: yup
      .string()
      .matches(/[\+]?[(]?[0-9]{3}[)]?[-\s\.]?[0-9]{3}[-\s\.]?[0-9]{4,6}/, {
        message: "Invalid Phone Number",
        excludeEmptyString: true,
      })
      .nullable(),
    permission_level: yup.string().required(),
    bio: yup.string().nullable(),
  });

  const {
    register,
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<yup.InferType<typeof formSchema>>({
    mode: "onTouched",
    resolver: yupResolver(formSchema),
    values: _.pick(user, [
      "email_verified",
      "phone_number",
      "fname",
      "lname",
      "bio",
      "profile_image",
      "permission_level",
    ]),
  });

  const { execute, status, error } = useAsync<
    yup.InferType<typeof formSchema>,
    CreateLocalUser
  >((data) => {
    let user_data = data as AdminUpdateUser;
    if (user_data.phone_number === "") {
      delete user_data.phone_number;
    }
    if (user_data.bio === "") {
      delete user_data.bio;
    }
    if (user_data.profile_image === "") {
      delete user_data.profile_image;
    }
    return axios.put(`/api/user/a/${user.id}`, user_data);
  });

  return (
    <>
      <form onSubmit={handleSubmit(execute)} className="">
        <div className="columns">
          <div className="column field mb-0">
            <label className="label">First Name*</label>
            <input
              type="text"
              className={classNames({
                input: true,
                "is-danger": errors.fname,
              })}
              placeholder="John"
              {...register("fname")}
            />
            {errors.fname && (
              <p className="help is-danger">{errors.fname.message}</p>
            )}
          </div>

          <div className="column field mb-0">
            <label className="label">Last Name*</label>
            <input
              type="text"
              className={classNames({
                input: true,
                "is-danger": errors.lname,
              })}
              placeholder="Doe"
              {...register("lname")}
            />
            {errors.lname && (
              <p className="help is-danger">{errors.lname.message}</p>
            )}
          </div>
        </div>

        <div className="field has-icons-left">
          <label className="label">Email*</label>
          <div className="control has-icons-left">
            <input
              type="text"
              className="input is-disabled"
              value={user.email}
            />
            <span className="icon is-small is-left">
              <FontAwesomeIcon icon={faEnvelope} width={16} />
            </span>
          </div>
        </div>

        <Controller
          control={control}
          name="email_verified"
          render={({ field: { onChange, value } }) => (
            <div className="field" onClick={() => onChange(!value)}>
              <input
                type="checkbox"
                className="switch"
                checked={value}
                onChange={onChange}
              />
              <label className="label">Email is Verified</label>
              {errors.email_verified && (
                <p className="help is-danger">
                  {errors.email_verified.message}
                </p>
              )}
            </div>
          )}
        />

        <div className="field">
          <label className="label">Phone Number</label>
          <div className="control has-icons-left">
            <input
              type="text"
              className={classNames({
                input: true,
                "is-danger": errors.phone_number,
              })}
              placeholder="(000) 000-000"
              {...register("phone_number")}
            />
            <span className="icon is-small is-left">
              <FontAwesomeIcon icon={faPhone} width={16} />
            </span>
          </div>
          {errors.phone_number && (
            <p className="help is-danger">{errors.phone_number.message}</p>
          )}
        </div>

        <div className="field">
          <label className="label">Permission Level*</label>
          <div className="control">
            <div className="select">
              <select {...register("permission_level")}>
                {Object.values(PermissionLevel).map((level) => (
                  <option key={level}>{level}</option>
                ))}
              </select>
            </div>
          </div>
          {errors.permission_level && (
            <p className="help is-danger">{errors.permission_level.message}</p>
          )}
        </div>

        <div className="field">
          <label className="label">Bio</label>
          <textarea
            {...register("bio")}
            rows={6}
            className={classNames({
              textarea: true,
              "is-danger": errors.bio,
            })}
            placeholder="Hi, my name is..."
          />
          {errors.bio && <p className="help is-danger">{errors.bio.message}</p>}
        </div>

        <p className="is-small is-pulled-left">*required</p>

        <div className="field is-grouped is-grouped-right mb-0">
          <div className="control">
            <button
              type="submit"
              className={classNames({
                button: true,
                "is-primary": true,
                "is-loading": status == AsyncStatus.Pending,
              })}
            >
              Update
            </button>
          </div>
          <div
            className="control has-tooltip-top has-tooltip-multiline"
            data-tooltip={
              user.permission_level == PermissionLevel.Admin
                ? "You cannot delete an admin without downgrading their permission level first."
                : undefined
            }
          >
            <button
              className="button is-danger"
              disabled={user.permission_level == PermissionLevel.Admin}
              type="button"
              onClick={() => setIsDeleteModalOpen(true)}
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
      </form>

      <ConfirmationModal
        isOpen={isDeleteModalOpen}
        setIsOpen={setIsDeleteModalOpen}
        actionName="Delete"
        onAccept={() => axios.delete(`/api/user/a/${user.id}`)}
        onSuccess={() => router.push("/settings/users")}
      >
        Are you sure you want to delete{" "}
        <strong>
          {user.fname} {user.lname}
        </strong>
        ?
      </ConfirmationModal>
    </>
  );
}

interface EditUserGroupsProps {
  userId: string;
}

function EditUserGroups({ userId }: EditUserGroupsProps) {
  const {
    execute,
    status,
    value: memberships,
    error,
  } = useAsync<void, MembershipData[]>(
    () => axios.get(`/api/user/a/${userId}/groups`),
    () => {},
    () => {},
    true
  );

  const [selectedMembership, setSelectedMembership] =
    useState<MembershipData | null>(null);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);

  return (
    <div>
      <h1 className="title is-4">Groups</h1>
      {(() => {
        if (status == AsyncStatus.Idle || status == AsyncStatus.Pending) {
          return <div className="page-loader"></div>;
        } else if (status == AsyncStatus.Error || !memberships) {
          return <p className="has-text-danger">{error?.message}</p>;
        } else if (memberships.length == 0) {
          return <p className="block">This user is not in any groups.</p>;
        } else {
          return (
            <table
              className="table"
              style={{ borderBottom: "solid #00000033 1px" }}
            >
              <thead>
                <th>Name</th>
                <th>Assigned Teacher</th>
                <th></th>
              </thead>
              <tbody>
                {memberships.map((membership) => (
                  <tr key={membership.group.id}>
                    <td>{membership.group.name}</td>
                    <td>
                      {membership.assigned_teacher
                        ? `${membership.assigned_teacher.fname} ${membership.assigned_teacher.lname}`
                        : "None"}
                    </td>
                    <td>
                      <p className="buttons">
                        <button
                          className="button is-primary is-light"
                          onClick={() => {
                            setSelectedMembership(membership);
                            setIsEditModalOpen(true);
                          }}
                        >
                          <FontAwesomeIcon
                            className="icon"
                            size="sm"
                            icon={faEdit}
                          />
                        </button>
                        <button
                          className="button is-danger is-light"
                          onClick={() => {
                            setSelectedMembership(membership);
                            setIsDeleteModalOpen(true);
                          }}
                        >
                          <FontAwesomeIcon
                            className="icon"
                            size="sm"
                            icon={faTrash}
                          />
                        </button>
                      </p>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          );
        }
      })()}

      {memberships && (
        <AsyncSearchSelect
          placeholder="Add group..."
          getList={() => axios.get("/api/group/a").then((res) => res.data)}
          getKey={(group: Group) => group.id}
          getName={(group) => group.name}
          excludeKeys={memberships.map(({ group }) => group.id)}
          onSelect={(group) => {
            axios.post(`/api/user/a/${userId}/groups/${group.id}`).then(() => {
              execute();
            });
          }}
        />
      )}

      {selectedMembership && (
        <>
          <ConfirmationModal
            isOpen={isDeleteModalOpen}
            setIsOpen={setIsDeleteModalOpen}
            actionName="Remove"
            onAccept={() =>
              axios.delete(
                `/api/user/a/${userId}/groups/${selectedMembership!.group.id}`
              )
            }
            onSuccess={() => execute()}
          >
            Are you sure you want to remove this user from{" "}
            <strong>{selectedMembership.group.name}</strong>?
          </ConfirmationModal>
          <EditMembershipModal
            isOpen={isEditModalOpen}
            setIsOpen={setIsEditModalOpen}
            userId={userId}
            membership={selectedMembership}
            onSuccess={() => execute()}
          />
        </>
      )}
    </div>
  );
}

interface AddGroupInputProps {
  existingGroupIds: number[];
  onSelect: (group: Group) => void;
}

function AddGroupInput({ existingGroupIds, onSelect }: AddGroupInputProps) {
  const {
    status,
    error,
    value: groups,
  } = useAsync<null, Group[]>(
    () => axios.get("/api/group/a"),
    () => {},
    () => {},
    true
  );

  const [searchInputValue, setSearchInputValue] = useState("");
  const [searchValueDebounced, setSearchValueDebounced] = useState("");
  useDebounce(() => setSearchValueDebounced(searchInputValue), 100, [
    searchInputValue,
  ]);

  // Hovering / highlighting
  const dropdownMenuRef = useRef<HTMLDivElement>(null);
  const dropdownMenuIsHovered = useHoverDirty(dropdownMenuRef);
  const [inputFocused, setInputFocused] = useState(false);

  const displayedGroups = useMemo(
    () =>
      groups &&
      groups
        .filter((group) => !existingGroupIds.includes(group.id))
        .filter((group) =>
          group.name.toLowerCase().includes(searchValueDebounced.toLowerCase())
        ),
    [groups, existingGroupIds, searchValueDebounced]
  );
  const [selectedId, setSelectedId] = useState(-1);

  useEffect(() => {
    if (
      !dropdownMenuIsHovered &&
      inputFocused &&
      searchValueDebounced.length > 0
    ) {
      setSelectedId(0);
    } else {
      setSelectedId(-1);
    }
  }, [dropdownMenuIsHovered, inputFocused, searchValueDebounced]);

  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <div
      className={classNames({
        dropdown: true,
        "is-active": inputFocused || dropdownMenuIsHovered,
      })}
    ></div>
  );
}

interface EditMembershipModalProps {
  isOpen: boolean;
  setIsOpen: Dispatch<SetStateAction<boolean>>;
  userId: string;
  membership: MembershipData;
  onSuccess: () => void;
}

function EditMembershipModal({
  isOpen,
  setIsOpen,
  userId,
  membership,
  onSuccess,
}: EditMembershipModalProps) {
  const [assignedTeacherId, setAssignedTeacherId] = useState<number | null>(
    membership.assigned_teacher?.id ?? null
  );

  return (
    <Modal isOpen={isOpen} setIsOpen={setIsOpen}>
      <div className="box"></div>
    </Modal>
  );
}
