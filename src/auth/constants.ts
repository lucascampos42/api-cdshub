if (!process.env.JWT_SECRET) {
  throw new Error('JWT_SECRET environment variable is not defined');
}

export const jwtConstants = {
  secret: process.env.JWT_SECRET,
};
