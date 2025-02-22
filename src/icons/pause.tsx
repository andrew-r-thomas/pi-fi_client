function Pause(props: { className: string; }) {
  const className = () => props.className;

  return (
    <svg
      class={className()}
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg">
      <path
        fill-rule="evenodd"
        clip-rule="evenodd"
        d="M10 4H5V20H10V4ZM19 4H14V20H19V4Z"
      />
    </svg>
  )
}

export default Pause;
