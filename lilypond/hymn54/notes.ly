melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'4 e4 a4 g4 c4 c4 b4 c4 \break

  % Line 2
  g4 c4 g4 a4 fis4 g2 \break

  % Line 3
  b4 c4 a4 d4 b4 c4 a4 b4 \break

  % Line 4
  g4 a4 c4 d4 b4 c2 \bar "|."
}
