function Play(props: { className: string; }) {
  const className = () => props.className;

  return (
    <svg
      class={className()}
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        fill-rule="evenodd"
        clip-rule="evenodd"
        d="M10 20H8V4H10V6H12V9H14V11H16V13H14V15H12V18H10V20Z"
      />
    </svg>
  )
}

export default Play;
