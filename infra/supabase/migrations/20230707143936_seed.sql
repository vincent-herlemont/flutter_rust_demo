CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

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
  FOR SELECT USING (true);
CREATE POLICY "Update own profile" ON public.profiles
  FOR UPDATE USING (auth.uid() = user_id);

CREATE OR REPLACE FUNCTION public.set_delete_time_on_user_delete()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE public.profiles
    SET deleted_at = NOW()
    WHERE id = OLD.id;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER delete_user_trigger
AFTER DELETE ON auth.users
FOR EACH ROW
EXECUTE PROCEDURE public.set_delete_time_on_user_delete();

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