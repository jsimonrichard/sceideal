import { useAsync, AsyncStatus } from "@/components/hooks";
import SettingsLayout, {
  AdminSettingPage,
  GeneralSettingPage,
} from "@/components/settingsLayout";
import { CreateLocalUser, PermissionLevel, UserData } from "@/shared-types";
import {
  faAdd,
  faEnvelope,
  faLock,
  faPhone,
} from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import axios, { isAxiosError } from "axios";
import classNames from "classnames";
import { Dispatch, ReactElement, SetStateAction, useState } from "react";
import { useForm } from "react-hook-form";
import { yupResolver } from "@hookform/resolvers/yup";
import { InferType, object, ref, string } from "yup";
import _ from "lodash";
import { useRouter } from "next/router";

export default function UsersSettings() {
  const router = useRouter();

  const [isCreateUserModalOpen, setIsCreateUserModalOpen] = useState(false);

  const {
    status,
    value: users,
    error,
    execute,
  } = useAsync<void, UserData[]>(
    () => axios.get("/api/user/a"),
    () => {},
    () => {},
    true
  );

  return (
    <div>
      <table className="table is-fullwidth is-striped is-hoverable">
        <thead>
          <tr>
            <th>ID</th>
            <th>Name</th>
            <th>Email</th>
            <th>
              <abbr title="Email Verified">Ver.</abbr>
            </th>
            <th>Phone Number</th>
            <th>
              <abbr title="Permission Level">Perm.</abbr>
            </th>
          </tr>
        </thead>

        <tbody>
          {(status == AsyncStatus.Idle || status == AsyncStatus.Pending) && (
            <tr>
              <td colSpan={6} className="loader-row"></td>
            </tr>
          )}
          {status == AsyncStatus.Error && (
            <tr>
              <td colSpan={6} className="has-text-danger">
                {error?.message}
              </td>
            </tr>
          )}
          {status == AsyncStatus.Success &&
            users &&
            users.length > 0 &&
            users.map((user) => (
              <tr
                key={user.id}
                onClick={() => {
                  router.push(`/settings/users/${user.id}`);
                }}
                style={{ cursor: "pointer" }}
              >
                <td>{user.id}</td>
                <td>
                  {user.fname} {user.lname}
                </td>
                <td>{user.email}</td>
                <td>{user.email_verified ? "Yes" : "No"}</td>
                <td>{user.phone_number || "-"}</td>
                <td>{user.permission_level}</td>
              </tr>
            ))}
          {status == AsyncStatus.Success && users && users.length == 0 && (
            <tr>
              <td colSpan={6} style={{ textAlign: "center" }}>
                No users have been created
              </td>
            </tr>
          )}
        </tbody>
      </table>

      <button
        className="button is-medium is-link is-inverted"
        onClick={() => setIsCreateUserModalOpen(true)}
      >
        <FontAwesomeIcon
          icon={faAdd}
          className="mr-3"
          style={{
            height: "1em",
          }}
        />
        <span>Create User</span>
      </button>

      <CreateUserModal
        isOpen={isCreateUserModalOpen}
        setIsOpen={setIsCreateUserModalOpen}
        onSuccess={execute}
      />
    </div>
  );
}

UsersSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={AdminSettingPage.Users}>{page}</SettingsLayout>
  );
};

interface CreateUserModalProps {
  isOpen: boolean;
  setIsOpen: Dispatch<SetStateAction<boolean>>;
  onSuccess?: () => void;
}

function CreateUserModal({
  isOpen,
  setIsOpen,
  onSuccess,
}: CreateUserModalProps) {
  const formSchema = object().shape({
    fname: string().required("First name is required"),
    lname: string().required("Last name is required"),
    email: string()
      .matches(/[\w\-\.]+@([\w\-]+\.)+[\w\-]{2,4}/, "Invalid Email")
      .required("Email is required"),
    phone_number: string()
      .matches(/[\+]?[(]?[0-9]{3}[)]?[-\s\.]?[0-9]{3}[-\s\.]?[0-9]{4,6}/, {
        message: "Invalid Phone Number",
        excludeEmptyString: true,
      })
      .nullable(),
    permission_level: string().required(),
    password: string()
      .required("Password is required")
      .min(4, "Password length should be at least 4 characters")
      .max(12, "Password cannot exceed more than 12 characters"),
    cpassword: string()
      .required("Confirm Password is required")
      .min(4, "Password length should be at least 4 characters")
      .max(12, "Password cannot exceed more than 12 characters")
      .oneOf([ref("password")], "Passwords do not match"),
  });

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<InferType<typeof formSchema>>({
    mode: "onTouched",
    resolver: yupResolver(formSchema),
  });

  const { execute, status, error } = useAsync<
    InferType<typeof formSchema>,
    CreateLocalUser
  >(
    (data) => {
      let user_data = data as CreateLocalUser;
      if (user_data.phone_number === "") {
        delete user_data.phone_number;
      }
      return axios.post("/api/user", user_data);
    },
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
          <h2 className="title">
            Create a User
            <span className="tag is-warning ml-4">Local-login users only</span>
          </h2>

          <form onSubmit={handleSubmit(execute)}>
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
                  className={classNames({
                    input: true,
                    "is-danger": errors.email,
                  })}
                  {...register("email")}
                />
                <span className="icon is-small is-left">
                  <FontAwesomeIcon icon={faEnvelope} width={16} />
                </span>
              </div>
              {errors.email && (
                <p className="help is-danger">{errors.email.message}</p>
              )}
            </div>

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
                <p className="help is-danger">
                  {errors.permission_level.message}
                </p>
              )}
            </div>

            <div className="field">
              <label className="label">Password*</label>
              <div className="control has-icons-left">
                <input
                  type="password"
                  className={classNames({
                    input: true,
                    "is-danger": errors.password,
                  })}
                  {...register("password")}
                />
                <span className="icon is-small is-left">
                  <FontAwesomeIcon icon={faLock} width={16} />
                </span>
              </div>
              {errors.password && (
                <p className="help is-danger">{errors.password.message}</p>
              )}
            </div>

            <div className="field">
              <label className="label">Verify Password*</label>
              <div className="control has-icons-left">
                <input
                  type="password"
                  className={classNames({
                    input: true,
                    "is-danger": errors.cpassword,
                  })}
                  {...register("cpassword")}
                />
                <span className="icon is-small is-left">
                  <FontAwesomeIcon icon={faLock} width={16} />
                </span>
              </div>
              {errors.cpassword && (
                <p className="help is-danger">{errors.cpassword.message}</p>
              )}
            </div>

            <p className="is-small is-pulled-left">*required</p>

            <div className="field is-grouped is-grouped-right">
              <div className="control">
                <button
                  type="submit"
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

            <p className="has-text-danger">
              {(error &&
                isAxiosError(error) &&
                typeof error.response?.data == "string" &&
                error.response.data) ||
                error?.message}
            </p>
          </form>
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
