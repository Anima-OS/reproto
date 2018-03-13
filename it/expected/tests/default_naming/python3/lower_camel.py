class Value:
  def __init__(self, foo_bar):
    self.foo_bar = foo_bar

  def get_foo_bar(self):
    return self.foo_bar

  @staticmethod
  def decode(data):
    f_foo_bar = data["fooBar"]

    return Value(f_foo_bar)

  def encode(self):
    data = dict()

    if self.foo_bar is None:
      raise Exception("fooBar: is a required field")

    data["fooBar"] = self.foo_bar

    return data

  def __repr__(self):
    return "<Value foo_bar:{!r}>".format(self.foo_bar)

