declare global {
    interface Window {
        [ key: string ]: any;
    }
}

export function registerFunction<T>(name: string, func: Function): (value: T) => void {

    let resolver: ((value: T) => void) | null = null

    window[ name ] = () => {

        func()

        return new Promise(resolve => resolver = resolve)

    }

    return value => resolver!(value)

}