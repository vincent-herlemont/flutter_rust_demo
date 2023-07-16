-- https://supabase.com/docs/guides/auth/row-level-security#testing-policies

-- https://supabase.com/blog/roles-postgres-hooks
grant anon, authenticated to postgres;

create or replace procedure auth.cache_set_uuid (user_email text)
    language plpgsql
    as $$
declare
    user_id uuid;
begin
    select id into user_id from auth.users where email = user_email;
    execute format('set cache.user_uuid=%L', user_id::text);
end;
$$;

CREATE OR REPLACE FUNCTION auth.cache_get_uuid()
RETURNS uuid
LANGUAGE plpgsql
AS $$
DECLARE
    saved_uuid uuid;
BEGIN
    -- Get the saved UUID from the session variable and cast it to uuid.
    SELECT CAST(CURRENT_SETTING('cache.user_uuid') AS uuid) INTO saved_uuid;
    RETURN saved_uuid;
END;
$$;

create or replace procedure auth.login_as_user (user_email text)
    language plpgsql
    as $$
declare
    auth_user auth.users;
begin
    select
        * into auth_user
    from
        auth.users
    where
        email = user_email;
    execute format('set request.jwt.claim.sub=%L', (auth_user).id::text);
    execute format('set request.jwt.claim.role=%I', (auth_user).role);
    execute format('set request.jwt.claim.email=%L', (auth_user).email);
    execute format('set request.jwt.claims=%L', json_strip_nulls(json_build_object('app_metadata', (auth_user).raw_app_meta_data))::text);

    raise notice '%', format( 'set role %I; -- logging in as %L (%L)', (auth_user).role, (auth_user).id, (auth_user).email);
    execute format('set role %I', (auth_user).role);
end;
$$;

create or replace procedure auth.login_as_anon ()
    language plpgsql
    as $$
begin
    set request.jwt.claim.sub='';
    set request.jwt.claim.role='';
    set request.jwt.claim.email='';
    set request.jwt.claims='';
    set role anon;
end;
$$;

create or replace procedure auth.logout ()
    language plpgsql
    as $$
begin
    set request.jwt.claim.sub='';
    set request.jwt.claim.role='';
    set request.jwt.claim.email='';
    set request.jwt.claims='';
    set role postgres;
end;
$$;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Profiles

CREATE TABLE public.profiles (
  id UUID DEFAULT uuid_generate_v4(),
  user_id UUID DEFAULT NULL UNIQUE REFERENCES auth.users ON DELETE SET NULL,
  first_name TEXT DEFAULT NULL,
  last_name TEXT DEFAULT NULL,
  deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
  PRIMARY KEY (id)
);
ALTER TABLE public.profiles ENABLE ROW LEVEL SECURITY;
CREATE POLICY "View all profiles" ON public.profiles
  FOR SELECT USING (true);
CREATE POLICY "Update own profile" ON public.profiles
  FOR UPDATE USING (auth.uid() = user_id)
  WITH CHECK (auth.uid() = user_id);
CREATE POLICY "Insert profile allow for supabase_auth_admin" ON public.profiles
  FOR INSERT WITH CHECK (CURRENT_USER = 'supabase_auth_admin');

CREATE OR REPLACE FUNCTION auth.set_delete_time_on_user_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE public.profiles
    SET deleted_at = NOW()
    WHERE profiles.user_id = OLD.id;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER delete_user_trigger
BEFORE DELETE ON auth.users
FOR EACH ROW
EXECUTE PROCEDURE auth.set_delete_time_on_user_delete();

CREATE OR REPLACE FUNCTION auth.create_profile_on_user_create()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO public.profiles (user_id) VALUES (NEW.id);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_profile_trigger
AFTER INSERT ON auth.users
FOR EACH ROW
EXECUTE PROCEDURE auth.create_profile_on_user_create();
GRANT INSERT ON public.profiles TO supabase_auth_admin;
GRANT EXECUTE ON FUNCTION auth.create_profile_on_user_create() TO supabase_auth_admin;

-- Runners
CREATE TYPE runner_status AS ENUM ('new','start', 'stop', 'unavailable');
CREATE TYPE runner_type AS ENUM ('hub', 'agent', 'file_transfer');

CREATE TABLE public.runners (
  id UUID DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL UNIQUE,
  type runner_type NOT NULL,
  uri TEXT NOT NULL,
  status runner_status DEFAULT 'new',
  total_available_memory_mo INTEGER DEFAULT 0,
  total_available_disk_mo INTEGER DEFAULT 0,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
  PRIMARY KEY (id)
);
ALTER TABLE public.runners ENABLE ROW LEVEL SECURITY;

-- Hubs
CREATE TYPE hub_status AS ENUM ('ready', 'busy', 'backing_up', 'restoring', 'unavailable');

CREATE TABLE public.hubs (
    id UUID DEFAULT uuid_generate_v4(),
    description TEXT DEFAULT NULL,
    runner_id UUID NOT NULL REFERENCES public.runners,
    profile_id UUID UNIQUE NOT NULL REFERENCES public.profiles,
    status hub_status DEFAULT 'unavailable',
    memory_consumption_mo INTEGER DEFAULT 0,
    disk_consumption_mo INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    PRIMARY KEY (id)
);
ALTER TABLE public.hubs ENABLE ROW LEVEL SECURITY;

CREATE VIEW public.hub_info AS
    SELECT h.*,
           r.uri as runner_uri,
           r.status as runner_status,
           r.total_available_memory_mo as runner_total_available_memory_mo,
           r.total_available_disk_mo as runner_total_available_disk_mo
    FROM public.hubs h
    JOIN public.runners r ON r.id = h.runner_id;

-- Hubs
CREATE TYPE agent_status AS ENUM ('ready', 'busy', 'unavailable');
CREATE TYPE agent_type AS ENUM ('google_drive', 'dropbox', 'onedrive', 's3', 'azure_blob', 'gcs');

CREATE TABLE public.agents (
    id UUID DEFAULT uuid_generate_v4(),
    runner_id UUID NOT NULL REFERENCES public.runners,
    profile_id UUID NOT NULL REFERENCES public.profiles,
    status agent_status DEFAULT 'unavailable',
    PRIMARY KEY (id)
);
ALTER TABLE public.agents ENABLE ROW LEVEL SECURITY;

-- File Transfers
CREATE TYPE file_transfer_status AS ENUM ('ready', 'busy', 'unavailable');

CREATE TABLE public.file_transfers (
    id UUID DEFAULT uuid_generate_v4(),
    runner_id UUID NOT NULL REFERENCES public.runners,
    profile_id UUID NOT NULL REFERENCES public.profiles,
    status file_transfer_status DEFAULT 'unavailable',
    PRIMARY KEY (id)
);
ALTER TABLE public.file_transfers ENABLE ROW LEVEL SECURITY;

-- Deployments
CREATE TYPE version_deployment_status AS ENUM ('scheduled', 'running', 'failed', 'deployed', 'finished');


CREATE TABLE public.version_deployments (
    id serial PRIMARY KEY,
    version varchar(20) CHECK (version ~ '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    status version_deployment_status DEFAULT 'scheduled',
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
ALTER TABLE public.version_deployments ENABLE ROW LEVEL SECURITY;
CREATE POLICY "View all version_deployments" ON public.version_deployments
  FOR SELECT USING (true);

CREATE OR REPLACE FUNCTION public.check_single_completed() RETURNS TRIGGER AS $$
BEGIN
   IF NEW.status = 'deployed' THEN
      IF EXISTS (SELECT 1 FROM public.version_deployments WHERE status = 'deployed') THEN
         RAISE EXCEPTION 'Only one version can be set to completed.';
      END IF;
   END IF;
   RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER check_single_completed_trigger
BEFORE UPDATE ON public.version_deployments
FOR EACH ROW EXECUTE PROCEDURE public.check_single_completed();

CREATE OR REPLACE FUNCTION check_scheduled_on_insert() RETURNS TRIGGER AS $$
BEGIN
   IF NEW.status <> 'scheduled' THEN
      RAISE EXCEPTION 'Only "scheduled" status is authorized when inserting.';
   END IF;
   RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_scheduled_on_insert
BEFORE INSERT ON version_deployments
FOR EACH ROW EXECUTE PROCEDURE check_scheduled_on_insert();

CREATE OR REPLACE FUNCTION check_version_status() RETURNS TRIGGER AS $$
DECLARE
    count INTEGER;
BEGIN
    IF TG_OP = 'INSERT' THEN
        SELECT COUNT(*)
        INTO count
        FROM public.version_deployments
        WHERE version = NEW.version AND status != 'failed';
    ELSIF TG_OP = 'UPDATE' THEN
        SELECT COUNT(*)
        INTO count
        FROM public.version_deployments
        WHERE version = NEW.version AND status != 'failed' AND id != NEW.id;
    END IF;

    IF count > 0 THEN
        RAISE EXCEPTION 'Version % is already in non-failed status', NEW.version;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER version_status_check_trigger
BEFORE INSERT OR UPDATE ON public.version_deployments
FOR EACH ROW EXECUTE PROCEDURE check_version_status();