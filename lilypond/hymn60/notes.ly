melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'4 a4 g4 f2 c4 f2 g4 a2 \break

  % Line 2
  c,4 f4 a4 c2 bes4 a4( g4) f4 g2 \break

  % Line 3
  e4 f4 g4 f2 f4 a4( bes4) c4 d2 \break

  % Line 4
  f,4 a4 g8 f2 \bar "|."
}
