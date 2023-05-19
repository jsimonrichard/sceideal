import { AsyncStatus, useAuth } from "@/components/hooks";
import { useForm } from "react-hook-form";
import { LoginData } from "../shared-types";
import { isAxiosError } from "axios";
import classNames from "classnames";
import { useRouter } from "next/router";

function Login() {
  const router = useRouter();
  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<LoginData>();
  const { user, login } = useAuth();
  const { execute, status, error } = login("/dashboard");
  const onSubmit = handleSubmit((data) => execute(data));

  return (
    <div className="p-2 center-vertically">
      <div className="box small-box">
        <form className="is-flex-grow-1" onSubmit={onSubmit}>
          <div className="field">
            <label className="label">Email</label>
            <div className="control">
              <input
                {...register("email", {
                  required: "This field is required",
                })}
                className="input"
                type="text"
                placeholder="johndoe@example.com"
                aria-invalid={errors.email ? "true" : "false"}
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
                  "is-primary": true,
                  "is-loading":
                    status == AsyncStatus.Pending ||
                    status == AsyncStatus.Success, // waiting for redirect
                })}
              >
                Login
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
  );
}

export default Login;
