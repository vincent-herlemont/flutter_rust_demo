-- https://supabase.com/docs/guides/auth/row-level-security#testing-policies

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
  deleted_at TIMESTAMP DEFAULT NULL,
  PRIMARY KEY (id)
);

ALTER TABLE public.profiles ENABLE ROW LEVEL SECURITY;

CREATE POLICY "View all profiles" ON public.profiles
  FOR SELECT TO authenticated USING (true);
CREATE POLICY "Update own profile" ON public.profiles
  FOR UPDATE TO authenticated USING (auth.uid() = user_id);

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

CREATE OR REPLACE FUNCTION public.create_profile_on_user_create()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO public.profiles (user_id)
    VALUES (NEW.id);

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_profile_trigger
AFTER INSERT ON auth.users
FOR EACH ROW
EXECUTE PROCEDURE public.create_profile_on_user_create();


-- Hubs
CREATE TYPE hub_status AS ENUM ('ready', 'busy', 'unavailable');

CREATE TABLE public.hubs (
  id UUID DEFAULT uuid_generate_v4(),
  description TEXT DEFAULT NULL,
  status hub_status DEFAULT 'unavailable',
  total_available_memory_mo INTEGER DEFAULT 0,
  total_available_disk_mo INTEGER DEFAULT 0,
  created_at TIMESTAMP DEFAULT NOW(),
  deleted_at TIMESTAMP DEFAULT NULL,
  PRIMARY KEY (id)
);


-- Hub Instances
CREATE TYPE hub_instances_status AS ENUM ('ready', 'busy', 'backing_up', 'restoring', 'unavailable');

CREATE TABLE public.hub_instances (
    id UUID DEFAULT uuid_generate_v4(),
    description TEXT DEFAULT NULL,
    status hub_instances_status DEFAULT 'unavailable',
    hub_id UUID DEFAULT NULL REFERENCES public.hubs ON DELETE CASCADE,
    profile_id UUID DEFAULT NULL REFERENCES public.profiles ON DELETE CASCADE,
    memory_consumption_mo INTEGER DEFAULT 0,
    disk_consumption_mo INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    deleted_at TIMESTAMP DEFAULT NULL,
    PRIMARY KEY (id)
);

