import { AsyncStatus, useAuth } from "./hooks";
import Image from "next/image";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faUser } from "@fortawesome/free-solid-svg-icons";
import Link from "next/link";
import { useState } from "react";
import classNames from "classnames";

export function NavBar() {
  const { user, logout, initialLoadStatus } = useAuth();
  const [isMenuActive, setIsMenuActive] = useState(false);
  const { execute: executeLogout, status: logoutStatus } = logout("/");

  return (
    <nav
      className="navbar is-fixed-top"
      role="navigation"
      aria-label="main navigation"
    >
      <div className="navbar-brand">
        <Link className="navbar-item" href="/">
          <Image
            src="/CPT_logo_with_text.svg"
            alt="logo"
            width={75}
            height={28}
          />
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
        <div className="navbar-start">
          {user && (
            <Link href="/dashboard" className="navbar-item">
              Dashboard
            </Link>
          )}
        </div>

        <div className="navbar-end">
          {(() => {
            if (
              initialLoadStatus == AsyncStatus.Idle ||
              initialLoadStatus == AsyncStatus.Pending
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
                <div className="navbar-item has-dropdown is-hoverable">
                  <a className="navbar-link">
                    <FontAwesomeIcon icon={faUser} className="mr-3" />{" "}
                    {user.username}
                  </a>

                  <div className="navbar-dropdown is-right">
                    <Link className="navbar-item" href="/dashboard#Profile">
                      Profile
                    </Link>
                    <Link className="navbar-item" href="/dashboard#Account">
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
              );
            }

            return (
              <div className="navbar-item">
                <div className="buttons">
                  <Link href="/login" className="button is-light">
                    Provider Login
                  </Link>
                </div>
              </div>
            );
          })()}
        </div>
      </div>
    </nav>
  );
}
