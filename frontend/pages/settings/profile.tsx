import { AsyncStatus, useAuth } from "@/components/hooks";
import SettingsLayout, {
  GeneralSettingPage,
} from "@/components/settingsLayout";
import { PermissionLevel } from "@/shared-types";
import { faPen } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import classNames from "classnames";
import { ReactElement } from "react";
import { useForm } from "react-hook-form";

export default function ProfileSettings() {
  const { user, initialLoadStatus } = useAuth();

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm({
    values: {
      fname: user?.fname,
      lname: user?.lname,
      bio: user?.bio,
    },
  });

  const onSubmit = handleSubmit((data) => console.log(data)); // TODO

  if (
    initialLoadStatus == AsyncStatus.Idle ||
    initialLoadStatus == AsyncStatus.Pending
  ) {
    return <div className="page-loader"></div>;
  }

  return (
    <div>
      <div className="media mb-4" style={{ alignItems: "center" }}>
        <div className="media-left">
          <figure className="image">
            <div className="profile-pic">
              {user?.fname[0]}
              {user?.lname[0]}
            </div>
          </figure>
        </div>
        <div className="media-content">
          <p className="title is-4">
            <span>
              {user?.fname} {user?.lname}
            </span>
          </p>
          <p className="subtitle is-6">{user?.email}</p>
        </div>
      </div>

      <form onSubmit={onSubmit}>
        <div className="columns" style={{ width: "max-content" }}>
          <div className="column field mb-0">
            <label className="label">First Name</label>
            <input type="text" className="input" {...register("fname")} />
          </div>

          <div className="column field mb-0">
            <label className="label">Last Name</label>
            <input type="text" className="input" {...register("lname")} />
          </div>
        </div>

        {user?.permission_level != PermissionLevel.Student && (
          <div className="field">
            <label className="label">Bio</label>
            <textarea
              {...register("bio")}
              rows={6}
              className="textarea"
              placeholder="Hi, my name is..."
            />
          </div>
        )}

        <div className="field mt-4 is-grouped">
          <div className="control">
            <button
              className={classNames({
                button: true,
                "is-link": true,
                "is-medium": true,
                // "is-loading":
                //   status == AsyncStatus.Pending ||
                //   status == AsyncStatus.Success, // waiting for redirect
              })}
            >
              Save
            </button>
          </div>
        </div>
      </form>
    </div>
  );
}

ProfileSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={GeneralSettingPage.Profile}>
      {page}
    </SettingsLayout>
  );
};
