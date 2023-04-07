import { AsyncStatus, useAsync, useAuth } from "@/components/hooks";
import { useForm } from "react-hook-form";
import { CreateUser, LoginData } from "../shared-types";
import axios, { AxiosError, isAxiosError } from "axios";
import classNames from "classnames";

function SignUp() {
  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<CreateUser>();
  const { execute: signUp, status, error } = useAuth().sign_up("/dashboard");
  const onSubmit = handleSubmit((data) => signUp(data));

  return (
    <div className="p-2 center-vertically">
      <div className="panel small-panel">
        <p className="panel-heading">Sign Up</p>
        <div className="panel-block">
          <form className="is-flex-grow-1" onSubmit={onSubmit}>
            <div className="field">
              <label className="label">Username</label>
              <div className="control">
                <input
                  {...register("username", {
                    required: "This field is required",
                  })}
                  className="input"
                  type="text"
                  placeholder="johndoe"
                  aria-invalid={errors.username ? "true" : "false"}
                />
              </div>
              {errors.username && (
                <p className="help is-danger">{errors.username.message}</p>
              )}
            </div>
            <div className="field">
              <label className="label">Email</label>
              <div className="control">
                <input
                  {...register("email", {
                    required: "This field is required",
                    pattern: {
                      value: /^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$/,
                      message: "Invalid email format",
                    },
                  })}
                  className="input"
                  type="text"
                  placeholder="johndoe@example.com"
                  aria-invalid={errors.password ? "true" : "false"}
                />
              </div>
              {errors.email && (
                <p className="help is-danger">{errors.email.message}</p>
              )}
            </div>
            <div className="field">
              <label className="label">Password</label>
              <div className="control">
                <input
                  {...register("password", {
                    required: "This field is required",
                  })}
                  className="input"
                  type="password"
                  placeholder="Password"
                  aria-invalid={errors.password ? "true" : "false"}
                />
              </div>
              {errors.password && (
                <p className="help is-danger">{errors.password.message}</p>
              )}
            </div>
            <div className="field is-grouped is-justify-content-end">
              <div className="control">
                <button
                  className={classNames({
                    button: true,
                    "is-link": true,
                    "is-loading":
                      status == AsyncStatus.Pending ||
                      status == AsyncStatus.Success, // waiting for redirect
                  })}
                >
                  Sign Up
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
    </div>
  );
}

export default SignUp;
