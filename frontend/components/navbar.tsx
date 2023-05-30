import { AsyncStatus, useAsync, useAuth, useConfig } from "./hooks";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faUser } from "@fortawesome/free-solid-svg-icons";
import Link from "next/link";
import { useEffect, useState } from "react";
import classNames from "classnames";
import axios from "axios";

export function NavBar() {
  const { user, logout, initialLoadStatus: initialAuthLoadStatus } = useAuth();
  const { config, initialLoadStatus: initialConfigLoadStatus } = useConfig();
  const [isMenuActive, setIsMenuActive] = useState(false);
  const { execute: executeLogout, status: logoutStatus } = logout();

  const {
    execute: getOIDUrl,
    status: OIDUrlStatus,
    value: OIDUrl,
  } = useAsync<null, string>(() =>
    axios.get(
      `/api/user/openid/${config?.oauth_providers.auth[0]}/generate_url`
    )
  );
  useEffect(() => {
    if (
      initialAuthLoadStatus == AsyncStatus.Error ||
      initialConfigLoadStatus == AsyncStatus.Success ||
      (config && config.redirect_to_first_oauth_provider)
    ) {
      getOIDUrl();
    }
  }, [initialAuthLoadStatus, initialConfigLoadStatus, config]);

  return (
    <nav
      className="navbar is-fixed-top"
      role="navigation"
      aria-label="main navigation"
    >
      <div className="container">
        <div className="navbar-brand">
          <Link className="navbar-item logo" href="/">
            <img src="/logo.svg" width="100%" height="100%" /> CPT Appointments
          </Link>

          <button
            role="button"
            className={classNames({
              "navbar-burger": true,
              "is-active": isMenuActive,
            })}
            aria-label="menu"
            aria-expanded="false"
            data-target="navbarBasicExample"
            onClick={() => setIsMenuActive(!isMenuActive)}
          >
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
          </button>
        </div>

        <div
          className={classNames({
            "navbar-menu": true,
            "is-active": isMenuActive,
          })}
        >
          {/* <div className="navbar-start"></div> */}

          <div className="navbar-end">
            {(() => {
              if (
                initialAuthLoadStatus == AsyncStatus.Idle ||
                initialAuthLoadStatus == AsyncStatus.Pending ||
                initialConfigLoadStatus == AsyncStatus.Idle ||
                initialConfigLoadStatus == AsyncStatus.Pending ||
                (config &&
                  config.redirect_to_first_oauth_provider &&
                  (OIDUrlStatus == AsyncStatus.Idle ||
                    OIDUrlStatus == AsyncStatus.Pending))
              ) {
                return (
                  <div className="navbar-item">
                    <div className="buttons">
                      <div className="button is-loading">Loading</div>
                    </div>
                  </div>
                );
              }

              if (user) {
                return (
                  <>
                    <Link href="/dashboard" className="navbar-item">
                      Dashboard
                    </Link>
                    <Link href="/classes" className="navbar-item">
                      Classes
                    </Link>
                    <Link href="/settings/profile" className="navbar-item">
                      Settings
                    </Link>
                    <div className="navbar-item has-dropdown is-hoverable">
                      <a className="navbar-link">
                        <FontAwesomeIcon
                          icon={faUser}
                          className="mr-3"
                          style={{
                            height: "1em",
                          }}
                        />{" "}
                        {user.fname} {user.lname}
                      </a>

                      <div className="navbar-dropdown is-right">
                        <Link className="navbar-item" href="/settings/account">
                          Account
                        </Link>
                        <hr className="navbar-divider" />
                        <a
                          className={classNames({
                            "navbar-item": true,
                            "is-loading": logoutStatus == AsyncStatus.Pending,
                          })}
                          onClick={() => executeLogout()}
                        >
                          Logout
                        </a>
                      </div>
                    </div>
                  </>
                );
              }

              let url;
              if (config && config.redirect_to_first_oauth_provider && OIDUrl) {
                url = OIDUrl;
              } else {
                url = "/login";
              }

              return (
                <div className="navbar-item">
                  <div className="buttons">
                    <Link href={url} className="button is-primary">
                      Login
                    </Link>
                  </div>
                </div>
              );
            })()}
          </div>
        </div>
      </div>
    </nav>
  );
}
