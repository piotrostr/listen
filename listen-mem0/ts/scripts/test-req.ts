const res = await fetch("http://localhost:9696/memories/search", {
  method: "POST",
  body: JSON.stringify({
    query: "what's my name",
    filters: {
      user_id: "some-user",
    },
  }),
});

console.log(res.status);

console.log(await res.json());
