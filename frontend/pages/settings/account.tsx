import { AsyncStatus, useAuth } from "@/components/hooks";
import SettingsLayout, {
  GeneralSettingPage,
} from "@/components/settings_layout";
import { ReactElement } from "react";

interface ProviderRecord {
  title: string;
  image: string;
}

const PROVIDER_LOOKUP: Record<string, ProviderRecord> = {
  keycloak: {
    title: "Keycloak",
    image: "https://www.keycloak.org/resources/images/keycloak_icon_512px.svg",
  },
};

export default function AccountSettings() {
  const { user, initialLoadStatus } = useAuth();

  if (
    initialLoadStatus == AsyncStatus.Idle ||
    initialLoadStatus == AsyncStatus.Pending
  ) {
    return <div className="page-loader"></div>;
  }

  return (
    <div>
      {user?.local_login != null && (
        <div className="field">
          <label className="label">Local Login</label>
          <button className="button is-link">Change Password</button>
        </div>
      )}

      {user?.oauth_providers.auth?.map((provider) => (
        <button className="button is-medium">
          <img
            style={{
              width: "1.5em",
              marginRight: "0.5em",
            }}
            src={PROVIDER_LOOKUP[provider.provider]?.image}
            alt={`${provider.provider} logo`}
          />
          {PROVIDER_LOOKUP[provider.provider]?.title || provider.provider}
        </button>
      ))}
    </div>
  );
}

AccountSettings.getLayout = (page: ReactElement) => {
  return (
    <SettingsLayout activePage={GeneralSettingPage.Account}>
      {page}
    </SettingsLayout>
  );
};
